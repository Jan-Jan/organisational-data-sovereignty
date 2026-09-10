# Verification — org-node risk analysis (2026-09-09)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line, so this file is the evidence that travels with the change.

branch: worktree-guardrails-org-node-risk
reviewer: seven rounds, each a fresh independent subagent reviewer dispatched at merge-change step 6a with the diff, the ledgers and the plan, and with no implementation narrative; each ran every unit's verify_commands itself in its own nested worktree rather than resting on the gate summary, and each verified the previous round's dispositions against the tree before reviewing afresh
verdict: PASS WITH FINDINGS in rounds 1 to 6 — eleven, seven, nine, eleven, ten and eight findings, fifty-six in all, every one dispositioned in a fix round and verified by the following round. Round 6 read "not ready to merge" and its findings were fixed. Round 7's verdict, findings and rankings are recorded in full under Review below, and are what this merge rests on
reproduced: the four defects this change files were reproduced by reading, not by running — PR-vt244s, PR-2dmjzj, PR-u4c2vp and PR-w88sr9 are each described from the code, with the reproducing test or the correction specified and deliberately not written, because this change is a hazard analysis and writing a fix under it would mix the two. Every code claim in all four was re-derived from the tree by the review round that followed, and the two the reviews themselves found were traced line by line in the round that found them. What was reproduced by running, and by mutation, is every requirement's test: each of the fifteen requirements was reddened by a named temporary change to the source under test, watched failing for that reason, reverted, and run green, with one assertion honestly declared green from birth and one shown to need a redesign of the type to fail. The regression a review found in this change itself — the panic-freedom lint broken by exposing the fixtures module — was reproduced directly: `cargo clippy -p org-node --lib --features app,test-support -- -D warnings` exited 101 with five errors before the fix and exits 0 after it, measured on both sides.

Change: org-node's first ISO 14971 hazard analysis, the fifteen requirements
realising its nine risk controls, the test evidence the traceability gate
reads, and three open problem reports. Branched from `master` at `89c3e26`.
Plan: `docs/plans/2026-09-09-org-node-risk-analysis.md`.
Units touched: org-members and org-node. Impact set (`check-units.sh --impact
master..HEAD`): org-members (touched), org-node (touched), app (dependent).

## The gate

Every figure below was measured by a dispatched gate subagent on the tree
this record travels with, and none is carried forward from an earlier run.
The gate ran seven times in all, once before each review round, and the last
run returned **PASS** with no gap of its own.

**The impact set is all four units, each `touched`** — org-members,
on-chain-client, org-node, app. It was three when the plan was written
(org-members and org-node touched, app dependent) and became four when a fix
round edited `.guardrails/units.yaml`: a change under the root `.guardrails/`
directory maps to every unit. Review round 5 caught the stale set, and the
last gate round is the first to cover on-chain-client. Logs were written
outside the tree.

| Gate | Result |
| --- | --- |
| `check-units.sh` (repository) | exit 0 — units 4, disclaimed 7, tracked paths 370 |
| `check-units.sh --impact master..HEAD` | all four units `touched` |
| `cargo test -p org-members` | 109 passed, 0 failed (integration 102, fuzz 6, mbt conformance 1) |
| `quint typecheck` (membership and protocol, both units) | exit 0, silent |
| on-chain-client's `--lib` plus three fuzz targets | 23 passed, 0 failed; three bolero no-panic runs |
| org-node's ten-target cargo line | **51 passed, 0 failed** — lib 23, verify_against_chain 13, store_at_rest 4, wire_frame_bound 3, admission_sender 3, service_stories 3, transport_handshake 1, transport_networked 1 |
| org-node's two bolero targets | no panic; about 211,000 and 213,000 generated inputs, each exiting on the one-second budget. `harness = false`, so no pass counts and not in the totals |
| `cargo test --manifest-path app/src-tauri/Cargo.toml` | compile proof, 0 tests (the crate has none) |
| `npm --prefix app run check` | 166 files, 0 errors, 1 warning (pre-existing: tsconfig cannot find the `node` type definitions) |
| `cargo clippy -p org-node --lib --features app,test-support -- -D warnings` | exit 0, 0 errors — the configuration review finding-2 showed broken |
| `cargo clippy -p org-node --lib --features app -- -D warnings` | exit 0, 0 errors |
| `check-ids.sh` per unit | exit 0 for all four; no draft tokens or draft-named files remain |
| `check-trace.sh` per unit | exit 0 for all four. org-node checked REQ 17, HAZ 6, RC 9, PR 4; org-members REQ 12, HAZ 8, RC 10, PR 1; on-chain-client and app all zero |
| Coverage, org-members | 93.50% statement against a floor of 92, 92.20% region against 91; both held |
| Coverage, on-chain-client | 53.57% statement against a floor of 52, 59.28% region against 58; both held, and both are the documented shortfall its config records rather than a class C target met |
| Coverage, decision | **no figure anywhere.** The branch column reads `0 0 -` for every file in both units, so the class C statement-AND-decision target is unmet |
| Coverage, org-node and app | no `coverage_command` — documented gaps |
| Working tree | clean |
| Worktrees registered inside the change worktree | none, after the last review worktree was removed at step 6a |

Suite total: **183 passed, 0 failed, 0 ignored** across the four units, plus
five bolero no-panic runs.

Both clippy lines were run after touching a source file, because a cached
lint pass is not a measurement. Neither line is in any unit's
`verify_commands`, so no configured gate runs it; that is under Gaps, since
it is why the review and not the gate caught the lint regression this change
introduced.

Warnings the gate reported, all expected and none blocking: two
UNMET-EXPECTATION lines for org-node's provider expectations (REQ-ysyu9g on
on-chain-client, REQ-q92yac on org-members, opened 2026-09-06, warnings until
2026-12-05), four UNRESOLVED-PR lines for org-node and one for org-members.
Both budgets hold in both units: `problem_open_max` is 10 against four open
in org-node and one in org-members, and the oldest item is well inside the
30-day age limit.

**Two judgements the gate asked the author to confirm.** First, six of the
fifteen requirements — REQ-wp2nyc, REQ-bvh8v6, REQ-gju89b, REQ-ag6kqm,
REQ-8gz8bu and REQ-6yu72z — are "shall reject" rules whose annotated tests
are all abnormal-input cases; the well-formed envelope that commits is
`happy_path_commits_when_root_matches_chain`, annotated to REQ-nhe2zu. That
is accepted as the correct shape: a rejection rule's subject is the abnormal
input, and the accept path is REQ-nhe2zu's own subject and is tested. Second,
a gate round judged REQ-uxv2x2's second test to be a well-formed input
with a contrasting outcome rather than an abnormal-input case, which earlier
rounds had accepted as the abnormal half. The stricter reading is right, and
the requirement gained a genuine abnormal-input case rather than an argument:
`unverified_revocation_leaves_the_record_in_place` delivers an envelope signed
by a key asserted not to be the Organisation's published signing key, from a
third device, and requires the rejection to leave the record's epoch, root,
sequence mark and member count untouched and the Persona active. The final
gate round confirms it as hostile input rather than a contrasting outcome, and
reports no requirement without an abnormal case.

That round named one further shape it declined to call a gap and thought
worth recording: the two fuzz requirements are annotated only on bolero
targets with no seed corpus, so the "or decode it" half of each is almost
certainly never reached by random bytes and rests on the happy-path test
annotated to REQ-nhe2zu. A one-envelope seed corpus would close them on their
own annotations; it is in the not-minted list and on the setup checklist
rather than in this change.

## Red → green

One row per ID this change implements, copied from the `red -> green:` lines
of the six dispatch reports (recorded as they landed in
`docs/plans/2026-09-09-org-node-risk-analysis.md`). These requirements are
worded from behaviour the crate already had, so a new test passes on first
run and a green run proves nothing: each was reddened by a **named temporary
mutation** of the source under test, watched failing for that reason,
reverted, and run green. No mutation was committed. The one exception is
stated as such.

| Item | Test | Watched red |
| --- | --- | --- |
| `REQ-wp2nyc` | `verify_against_chain::rejects_root_mismatch_when_chain_root_differs` | step 8 made to verify against the candidate's own root instead of the chain's; failed with `unwrap_err()` on an `Ok(VerifiedUpdate { .. })` |
| `REQ-bvh8v6` | `verify_against_chain::rejects_when_org_absent_from_chain` | `.ok_or(OrgNotOnChain)?` replaced by a fabricated `OrgState` at epoch 99; failed with left `RootMismatch`, right `OrgNotOnChain` |
| `REQ-gju89b` | `verify_against_chain::rejects_wrong_org_id`; `rejects_wrong_org_before_decoding_delta` | step 1's `if` deleted (left `OrgNotOnChain`, right `OrgIdMismatch`); and `decode_delta()?` moved ahead of step 1 (left `MalformedDelta`, right `OrgIdMismatch`) |
| `REQ-ag6kqm` | `verify_against_chain::rejects_bad_signature`; `rejects_bad_signature_before_decoding_delta` | step 2's `if` deleted (left `OrgNotOnChain`, right `BadSignature`); and `decode_delta()?` moved between steps 1 and 2 (left `MalformedDelta`, right `BadSignature`) |
| `REQ-8gz8bu` | `verify_against_chain::rejects_stale_epoch` | step 7's `<=` changed to `<`; failed with `unwrap_err()` on an `Ok(VerifiedUpdate { .. epoch: 1 .. })` |
| `REQ-nhe2zu` | `verify_against_chain::happy_path_commits_when_root_matches_chain`; `service_stories::five_stories_full_e2e` | `seq_guard.advance(envelope.parent_seq)` deleted; failed with `last_seen()`: left 1, right 2 |
| `REQ-6yu72z` | `verify_against_chain::rejects_stale_seq`; `rejects_equal_and_lower_seq`; `rejects_stale_seq_before_decoding_delta` | step 3's `seq_guard.check(...)?` deleted (`unwrap_err()` on an `Ok(..)` at `last_seen: 2`); `SeqGuard::check`'s `>` changed to `>=` (left `Ok(())`, right `Err(StaleSeq { got: 5, last_seen: 5 })`); `decode_delta()?` moved between steps 2 and 3 (left `MalformedDelta`, right `StaleSeq { got: 1, last_seen: 1 }`) |
| `REQ-mr5abb` | `verify_against_chain::advance_moves_high_water_mark_forward_only`; `check_does_not_advance_the_mark` | `SeqGuard::advance`'s guard `if` deleted so it always assigns; failed with left 2, right 3. **`check_does_not_advance_the_mark` was green from birth** and is declared so: `check` takes `&self`, so no mutation through a shared reference can move the mark; the test documents the contract the type system enforces |
| `REQ-9g6as6` | `fuzz_envelope_decode` (bolero, `harness = false`) | not reddened by mutation: the target predates this change and was already running at every merge; what this change adds is its `verifies:` annotation. Its red condition is a panic, which is the bolero failure signal |
| `REQ-bcxz96` | `fuzz_verify_against_chain` (bolero, `harness = false`) | as above; the target also asserts that any accepted update carries the chain's root |
| `REQ-eg5j8u` | `wire_frame_bound::frame_round_trips`; `oversize_body_is_rejected`; `oversize_message_is_rejected_on_encode` | length prefix switched to big-endian in `encode_frame` only (left 184614912, right 267); the `MAX_FRAME` check deleted in `decode_body`; the same check deleted in `encode_frame` — each failed its own assertion |
| `REQ-xa6smf` | `admission_sender::first_admission_from_a_device_other_than_the_invites_admin_is_rejected`; `service_stories::five_stories_full_e2e` | the `return Err(BadSignature)` inside the first-admission invite check deleted; failed with `B must reject a first admission whose sender is not the invite's admin device, got Ok(ReceiveOutcome { epoch: 2, .. })` |
| `REQ-ztdza4` | `admission_sender::update_from_the_admin_after_admission_is_committed`; `update_relayed_by_a_non_member_after_admission_is_rejected` | `if !sender_known` inverted (failed with `B must accept an update sent by the admin's own device: BadSignature`); and the `return Err` inside the `sender_known` block deleted (failed with `B must reject an update whose sender is not a member device, got Ok(ReceiveOutcome { epoch: 3, .. })`) |
| `REQ-uxv2x2` | `service_stories::five_stories_full_e2e` (story 5); `revocation_of_another_member_is_committed_not_self_deleted`; `unverified_revocation_leaves_the_record_in_place` | `my_still_present` forced to `false` in `receive_and_self_delete_if_revoked`; failed with `B must not self-delete when a different member is revoked`. Story 5 stays green under that mutation — it asserts `SelfDeleted`, which the forced `false` produces — which is why the second test was added. The third is the abnormal-input case a later gate round found missing: verification's `?` replaced by a `match` whose `Err(_)` arm ran the self-delete branch and returned `Ok(SelfDeleted)`; failed with `a revocation whose envelope fails verification must be rejected: SelfDeleted { org_id: … }`; reverted; green |
| `REQ-hzm4kt` | `store_at_rest::round_trips_encrypted_through_disk`; `wrong_passphrase_yields_error_not_data`; `seeds_do_not_appear_in_the_file` | `save` made to write plaintext instead of ciphertext (failed with `member seed in clear`, and the round trip failed at the correct-passphrase reopen with `decrypt failed`); and `open`'s decrypt-error branch replaced by an empty store (failed with `must fail`, open having returned `Ok`) |

## What was wrong, and what was built

**What was wrong.** org-node held the keys, made the trust decisions and did
the chain and p2p I/O through which an Organisation's access is granted or
withdrawn, and it had no risk analysis of its own. Its hazards lived as eight
mentions inside org-members' register, written from the membership
capability's point of view; an org-node defect (the chain reader's finality
doc-comment) was filed in org-members' problems ledger because that was the
only ledger when it was found; and its ledgers held nothing but the two
dependency expectations from tooth 2. Under IEC 62304 class C that is a unit
with mandatory risk management unperformed.

Worse, and measured on this change: the behaviour that makes org-node safe
was tested and the evidence counted for nothing. Nine unit tests inside
`org-node/src` exercised the verify-and-commit order, the replay guard, the
frame bound and the store's encryption, and not one carried a `verifies:`
annotation — `test_paths` points at `org-node/tests`, so no gate ever read
them. org-members' own register had named this exact shape of gap for the
fuzz targets ("evidence that exists and counts for nothing") and it applied
to the whole unit.

**What was built.** A hazard register for the unit
(`org-node/docs/risk/2026-09-09-org-node-hazards.md`): six hazards with
identifiers and nine risk controls, seven further hazards in prose with their
severities and their not-minted controls, a per-hazard residual-risk table,
thirteen not-minted controls, and a derived-requirements assessment of every
requirement this change mints. The six minted hazards are a Change set
accepted on its sender's word, a superseded envelope applied again, the node
brought down by hostile input, membership material handed to or taken from a
device the record does not name, a device removed from the record that keeps
acting, and keys read from the device's storage.

Fifteen requirements realise the nine controls
(`org-node/docs/requirements/2026-09-09-verify-and-commit.md`), each worded
from behaviour the crate already had — no source behaviour changed in this
change — and each `satisfies: derived`, assessed in the register.

The evidence moved to where the gate reads it. The nine unit tests were
relocated into four annotated integration targets
(`verify_against_chain`, `wire_frame_bound`, `store_at_rest`,
`admission_sender`), their `#[cfg(test)]` modules deleted from `verify.rs`,
`sequence.rs`, `transport/wire.rs` and `store.rs`, and the class C
abnormal-input cases added beside them: the three "before decoding the delta"
ordering tests, the encode-side frame bound, the wrong passphrase, a
plaintext scan of the store file, a rogue relay on first admission, a rogue
relay after admission, the admin's own update accepted, and a revocation of
another member committed rather than self-deleted. `test_fixtures` became
`pub mod` under the `test-support` feature so the integration tests can build
the same deterministic tries; the feature is never enabled in a production
build. The two fuzz targets and `service_stories` gained annotations. Four new
`[[test]]` entries went into `org-node/Cargo.toml`, into org-node's
`verify_commands`, and into the org-node step of `rust.yml`.

Five problem reports opened org-node's problems ledger
(`org-node/docs/problems/2026-09-09-org-node-problems.md`). PR-hvg2dy moved
from org-members with its original text and opened date and its `affects:`
retargeted from an org-members control to RC-6a2dke, because a consumer may
not cite a provider's controls; org-members' problems file keeps a dated
pointer and defines no item, and the one line of org-members' register that
named the report by ID now names the file, because a provider may not cite a
consumer's items. PR-vt244s and PR-2dmjzj are the two defects the code survey
found: the publish path writes the chain before it persists the record, and
loopback admission delivers the envelope and the Organisation secret to an
address it never bound to the joiner's Device key. The reviews then found two
more — PR-u4c2vp, the revocation receive path committing a record from a
sender it never checks, and PR-w88sr9, a wire-message field whose doc-comment
contradicts the send path this register assesses. All four are recorded as
hazards or as the evidence for one, with their fixes in the not-minted list;
none is fixed here, because this change is a hazard analysis and touching
`service.rs` or its comments under it would mix the two. That is the same
call org-members' register made for its own unimplemented clause, and the
same line this change drew for the report it inherited.

The unit glossary (`org-node/docs/CONTEXT.md`) was filled with the internal
terms the requirements use — Envelope, Wire message, Sequence number,
Published signing key, Persona, Persona store, Organisation secret, Invite,
Join request, Receive operation — leaving the interface vocabulary in the root
glossary where tooth 2 put it. Two of those entries exist because reviews
found the requirements leaning on words the glossary either forbade or had not
defined: *Wire message* names what a device actually sends, which is an
Envelope plus, on both send paths, the Organisation secret and a membership
snapshot outside the signature, and *Receive operation* had to be widened to
it and split into the two operations the node performs.

**The conclusion the register reaches.** Against the intended use that makes
this software class C, the overall residual risk of the org-node unit is
UNACCEPTABLE, and **no assessed risk in the register is acceptable**. Twelve
distinct hazards, every one S3: five open defects, one control whose decisive
half is an open expectation on a provider, one hazard whose remaining residual
waits on a second provider, one control that depends on the excluded device's
cooperation, a store whose protection is a passphrase over a fixed salt, and
three hazards the controls themselves introduce.

The last of those is the change's own last correction. The register carried
one hazard it called acceptable with no control needed — a spurious
self-delete on an inconsistent store — until the sixth review round showed the
severity had been argued from recoverability, which this project's class C ADR
withdrew in as many words on 2026-09-02. Re-assessed S3/P1 and not
acceptable, it left the register with no exception, which is what a unit
holding the keys and carrying five open defects should look like on paper.

That restates for this unit what org-members' register concluded on 2026-09-02
for the membership capability, and it does not change the class, which is set
by the planned intended use.

## Review

Round 1, on `53862bc`: **PASS WITH FINDINGS, eleven.** The reviewer ran every
unit's `verify_commands` itself in its own nested worktree and reported the
counts it saw (org-node 49 passed, org-members 109 passed, app compile proof
and 166 files with 0 errors), ran all four guardrails gates per unit, and in
addition ran a check no configured gate runs — `cargo clippy -p org-node
--lib --features app,test-support -- -D warnings` — which is how finding-2
was found. Its findings are copied below in its own words, each with the
author's disposition. Every one was dispositioned in fix round 1
(`.worktrees/worktree-guardrails-org-node-risk-fix1`, merged onto the change
branch), after which the sequence reran from step 1.

**finding-1**: REQ-ztdza4 is worded as unconditional behaviour that the code satisfies on only one of the two receive paths. `org-node/docs/requirements/2026-09-09-verify-and-commit.md:103-108` requires that "for a message about an Organisation it already holds a record of" the software reject the message when the authenticated Device key is not in the verified record. `receive_and_verify` does this (`org-node/src/service.rs:1018-1027`), but `receive_and_self_delete_if_revoked` handles exactly that class of message, explicitly discards the authenticated sender (`org-node/src/service.rs:1337`, `let _ = remote_device_key;`) and, in the `UpdatedNotRevoked` branch, verifies and commits a record from an unchecked sender (`:1339-1365`). RC-b6mydy's own text (`org-node/docs/risk/2026-09-09-org-node-hazards.md:291-297`) makes the same unconditional claim, "for every later message about that Organisation". The register discloses the gap as RC-b6mydy weakness 2 (`:316-322`) and as not-minted control 4 (`:624-626`), but neither the requirement nor the control text is scoped to the implementing path, and both tests annotated `verifies: REQ-ztdza4` (`org-node/tests/admission_sender.rs:232`, `:264`) exercise only `receive_and_verify`, so no test can detect the gap. Either the REQ and RC must be scoped to the path that implements them, or the unchecked branch needs a problem report rather than a not-minted-control bullet.
disposition: accepted, and both halves of the reviewer's alternative were taken, because they answer different objections. REQ-ztdza4 and RC-b6mydy's second clause are now scoped to the admission-and-update receive operation, so the requirement is true as written of the code and the two tests annotated to it verify exactly the path they exercise. And the unchecked branch is a defect, not merely an unimplemented improvement, so it is filed as **PR-u4c2vp** (`affects: RC-b6mydy`, opened 2026-09-09, open), named in the control's weakness paragraph and in not-minted control 4. No test reddens without this change, because it moves a boundary in prose rather than in code; what would redden is a test of the revocation path's sender check, which is the fix PR-u4c2vp specifies and this change deliberately does not make.

**finding-2**: exposing `test_fixtures` under `test-support` puts five `unwrap()` calls into library code and breaks the crate's own panic-freedom deny-lint in the configuration every gate uses. `CARGO_HOME=/tmp/cargo_home_fuzz cargo clippy -p org-node --lib --features app,test-support -- -D warnings` fails with 5 errors at `org-node/src/test_fixtures.rs:23`, `:42` (×2) and `:55` (×2), each attributed to `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` at `org-node/src/lib.rs:2`; the same command with `--features app` alone is exit 0 with no errors, so this is a regression introduced by the `#![cfg(any(test, feature = "test-support"))]` / `pub mod` change. `--features app,test-support` is precisely what `org-node/.guardrails/config.yaml` and `.github/workflows/rust.yml:57` compile `--lib` under. It escapes CI only because the `clippy` job covers just org-members and on-chain-client (`.github/workflows/rust.yml:86-87`). It also falsifies the register's evidence sentence for RC-e2uvje — "the crate denies `unwrap`, `expect` and `panic` … and the survey found none in non-test code" (`org-node/docs/risk/2026-09-09-org-node-hazards.md:249-253`) — since `test_fixtures.rs` is now non-test code in the tested configuration. The other new test files carry `#![allow(clippy::unwrap_used, clippy::expect_used)]`; `test_fixtures.rs` did not get one.
disposition: accepted; the most serious finding of the round, and a regression this change introduced. `org-node/src/test_fixtures.rs` now carries `#![allow(clippy::unwrap_used, clippy::expect_used)]` with a doc sentence stating why the allow is deliberate and bounded: the module holds deterministic fixture constructors, is compiled only under `cfg(test)` or the `test-support` feature, is never in a production build, and the crate's denial continues to govern every shipped module. Both clippy invocations now exit 0 and the figures are in the gate table. The register's RC-e2uvje evidence sentence was corrected to say exactly that rather than "none in non-test code". The gap in CI's clippy job coverage — org-node is not in it — is recorded under Gaps.

**finding-3**: the target annotated `verifies: REQ-bcxz96` cannot detect half of what that REQ claims. `org-node/tests/fuzz_verify_against_chain/fuzz_target.rs:46` seeds the chain state with `local.root_hash()` — the *unchanged* local root — and `:47`/`:61` supply the honest admin's verifying key. Reaching the accept branch therefore requires fuzzed bytes that decode as a `SignedDeltaEnvelope`, carry `org_id == [5u8;20]`, carry a valid ed25519 signature by the admin's key, and apply to a candidate whose root equals the pre-change root. The `assert_eq!` at `:67-70` is unreachable in practice, so the target verifies only the "shall not panic" clause of REQ-bcxz96 (`…verify-and-commit.md:81-86`) and not "commit an update whose Membership root equals that Organisation state's root". The register nevertheless describes the target as "asserting that anything accepted has the chain's root" (`…hazards.md:255-258`), and the derived assessment calls REQ-bcxz96 "the property form of the root match" (`:692-695`).
disposition: accepted. REQ-bcxz96 is narrowed to the panic-freedom property of verification over arbitrary envelope bytes, and its annotation is now `(implements: RC-e2uvje)` alone; RC-6a2dke keeps REQ-wp2nyc and REQ-bvh8v6, so no control is left unimplemented, which `check-trace.sh` confirms. The register's description of the target and the derived assessment of REQ-bcxz96 were both corrected, and they now say that the root-match property is established by `rejects_root_mismatch_when_chain_root_differs` and `happy_path_commits_when_root_matches_chain`. The reviewer's point that a fuzz target's unreachable assertion is not evidence is the same correction org-members' register had to make about its own fuzz claims.

**finding-4**: PR-hvg2dy names the wrong method as the site of the defect. `org-node/docs/problems/2026-09-09-org-node-problems.md:16-20` states that `OnChainReader::get_org_state` "is documented as fetching 'the latest (current best) state', while the read it performs passes `at = None`". In the tree, `OnChainReader::get_org_state` is `org-node/src/chain_read.rs:50-59`, is documented (`:51-53`) as returning the cached snapshot and failing closed, and performs no read at all; the "Fetch the latest (current best) state" doc-comment is on `refresh()` at `:34-35`, and the `get_org_state(admin, None)` call is at `:40`. The report's own third paragraph (`:43-48`) contradicts its first on this point. The underlying defect is real — `on-chain-client/src/client.rs:145-146` documents `at = None` as the latest finalised block — but the description is wrong about which method carries it, and the register repeats the phrasing at `…hazards.md:174-176`. Keeping the 2026-09-02 wording verbatim was the stated intent, yet the move is the point at which the description had to be checked against the tree, and the change's own notes-at-the-move paragraph (`:62-71`) revised other facts without correcting this one.
disposition: accepted, and the reviewer's reasoning about *why* it is this change's problem is the part worth keeping: carrying a description across a unit boundary is exactly the moment to re-verify it, and this change had already re-verified other facts in the same paragraph. The 2026-09-02 body stands, per the project's convention of dated corrections over rewrites, and the notes-at-the-move paragraph now carries a dated correction naming `refresh()` as the method whose doc-comment is wrong and the line the `at = None` call is on. The register's repetition was corrected in place. The defect is unchanged in substance and the report stays open.

**finding-5**: five `file:line` citations in the register are off by one or exclude the fact they are offered for, against the plan's claim that all 42 ranges were re-checked and confirmed (`docs/plans/2026-09-09-org-node-risk-analysis.md:62-63`). At `org-node/docs/risk/2026-09-09-org-node-hazards.md:210-212`: `org-node/src/verify.rs:62` is cited for the `SeqGuard::check` call, but `:61` is the call and `:62` is the `// 4. Decode the delta` comment; `:83-84` is cited for `SeqGuard::advance`, but `:83` is blank, `:84` copies the guard, and `advance` is at `:85`; `org-node/src/sequence.rs:32-48` is cited for "`check` … does not mutate", but `check`'s `&self` signature is at `:28`, outside the range. Additionally `org-node/src/service.rs:418-427` (`…hazards.md:169-170`) ends on the impl-block brace rather than covering `read_state`, which is `:417-426`; and `org-node/src/chain_read.rs:33-34` (`:175-176`) starts on a blank line.
disposition: accepted, all five corrected against the tree. The finding is also a correction to this change's own record of its work: the plan claimed 42 ranges re-checked and confirmed, and that check plainly missed five, so the plan now says what actually happened rather than what was intended. The reviewer's enumeration of the forty-odd citations it checked and found sound is why this is a precision finding and not a credibility one.

**finding-6**: REQ-hzm4kt bundles three properties and one of them is untested. Its third clause — "no member seed, device seed or Organisation secret appears in the file in clear" (`…verify-and-commit.md:119-124`) — is verified by `seeds_do_not_appear_in_the_file` (`org-node/tests/store_at_rest.rs:73-85`), which pushes only a `PersonaRecord` and searches the ciphertext for the two seeds and the handle. The Organisation secret lives in `OrgRecord.org_secret` (written at `org-node/src/service.rs:1083`, `:1091`); no test in the suite writes one into a store and searches the file for it, so that clause has no evidence. The bundling itself is against the ledger grammar's "One requirement per item" (`org-node/docs/requirements/README.md`), as is REQ-eg5j8u's "on send and on receive alike, and shall deliver a frame within that bound unchanged".
disposition: accepted on the evidence half, and answered rather than accepted on the grammar half. The missing evidence is now `organisation_secret_does_not_appear_in_the_file` in `org-node/tests/store_at_rest.rs`, which writes an `OrgRecord` carrying a known 32-byte secret, saves, and asserts the secret's bytes are absent from the file; it was reddened by making `save` write plaintext. On the bundling: both items state one behaviour about one artefact — what the store file may not contain, and what a frame over the bound must not do — enforced at more than one place in the code, and splitting them per key or per direction would multiply items without adding a testable distinction. The reviewer is right that this is a judgement call against the grammar's letter; it is recorded here as one.

**finding-7**: PR-2dmjzj misquotes the probability of the hazard it cites, and the register's hazard count is inflated. `org-node/docs/problems/2026-09-09-org-node-problems.md:131-132` says "HAZ-ep6uzs in org-node's register, S3/P1", but HAZ-ep6uzs is assessed S3/**P2** (`…hazards.md:285-286`); the S3/P1 belongs to the prose sub-hazard at `…hazards.md:512-518`. Separately, the conclusion "of thirteen hazards, none carries a residual risk this project's matrix calls acceptable" (`:585-586`) reaches thirteen by adding seven prose hazards to the six identified ones, while the register itself says two of those seven are restatements of HAZ-vxabf9 (`:486-487`) and HAZ-ep6uzs (`:512-513`). The acceptability conclusion is unaffected — every entry is S3, which the matrix in `org-node/docs/risk/README.md` makes UNACCEPTABLE at every probability, and the two S2/P1 control-introduced hazards are correctly called acceptable — but the denominator double-counts.
disposition: accepted, both corrected. PR-2dmjzj now cites HAZ-ep6uzs at S3/P2 and attributes the S3/P1 to the prose sub-hazard about the admission address, which is the one it is actually about. The residual-risk conclusion now counts six identified hazards and five further distinct hazards in prose, naming the two prose entries that restate identified ones. The conclusion itself is unchanged, for the reason the reviewer gives: every entry is S3 and the matrix makes S3 unacceptable at every probability, so the verdict never depended on the denominator.

**finding-8**: PR-vt244s names a risk control it never connects to the defect. `affects: RC-wqgm2p, RC-b6mydy` (`org-node/docs/problems/2026-09-09-org-node-problems.md:81`), but the report's "Where", "Observable symptom" and "Why it is a hazard" paragraphs (`:85-101`) argue only the undelivered revocation (RC-wqgm2p) and the administrator's subsequent inability to publish. Nothing explains how writing the chain before the record degrades RC-b6mydy's sender cross-check, and the register's publish-before-persist prose (`…hazards.md:455-468`) does not either.
disposition: accepted; `affects:` is now `RC-wqgm2p` alone. An `affects:` line is a claim about which controls the defect degrades, and a control listed without an argument makes the field decorative.

**finding-9**: REQ-nhe2zu is the one claimed ID with no abnormal-input case of its own. Its two tests — `happy_path_commits_when_root_matches_chain` (`org-node/tests/verify_against_chain.rs:61`) and `five_stories_full_e2e` (`org-node/tests/service_stories.rs:37`) — are both normal cases. Its negative space is covered by REQ-wp2nyc, REQ-bvh8v6, REQ-8gz8bu, REQ-6yu72z, REQ-gju89b and REQ-ag6kqm, each of which does have an abnormal case, so no behaviour is untested; but the ID itself carries none.
disposition: accepted. `rejects_root_mismatch_when_chain_root_differs` now carries `verifies: REQ-wp2nyc, REQ-nhe2zu`, with a comment stating why: REQ-nhe2zu is a conditional commit rule, so a case that withholds the commit when one condition fails is its abnormal-input case, and this is the condition the rule exists for. The test's existing red-by-mutation attestation (step 8 made to verify against the candidate's own root) covers both IDs, since the mutation makes the rule commit where it must not.

**finding-10**: the glossary this change filled restates a requirement and is partly duplicated by the requirements file. `org-node/docs/CONTEXT.md` opens its Language section with "Definitions only — no implementation details, no specs", yet the **Sequence number** entry states the commit rule ("A receiving node commits Envelopes in strictly increasing Sequence-number order and refuses one at or below the highest it has committed") — which is REQ-6yu72z, now asserted in two places that must be kept in step. The requirements file also re-defines "The published signing key" inline (`…verify-and-commit.md:15-16`) instead of citing the **Published signing key** entry the same change added; and REQ-uxv2x2 uses "persona" in lower case while the Terms paragraph omits **Persona** from the glossary terms it lists.
disposition: accepted, all three corrected. The **Sequence number** entry is now a definition of what the value is; the requirement it restated is REQ-6yu72z's alone. The requirements file points at the glossary's **Published signing key** entry instead of re-defining it, and its Terms paragraph now names **Persona**. The reviewer's underlying rule is the one the glossary's own header states, and a definition that carries a rule is a second copy of that rule.

**finding-11**: substantial unit behaviour has no requirement, though the change's scope claims the whole unit. The register scopes itself to "every module of `org-node/src`" (`…hazards.md:16-19`), and the fifteen REQs cover verification, replay, the frame bound, the two sender checks, self-delete and the store. `ceremony.rs`, `keys.rs`, `blobs.rs` encode/decode, `chain_write/` dispatch, `preflight.rs`, the transport handshake and the whole publish path have behaviour with no REQ and no LLR. The requirements file's "What these requirements do not say" (`:126-138`) names only four of these omissions. This is mechanically clean — `check-trace.sh` reports no untraced design and the architecture/LLR work is tooth 4 — but under class C it is unmarked derived behaviour, and the register's "whole unit" framing reads as broader coverage than the requirements deliver.
disposition: accepted as a framing correction, not as a scope change. The register's scope section now distinguishes the two things the change does: the hazard analysis covers the whole unit, and the requirements minted here realise the nine controls this analysis chose. The requirements file's "What these requirements do not say" now names the modules with no requirement — the genesis ceremony, key handling, the blob encoding, chain-write dispatch, preflight and the transport handshake — and points at tooth 4, where the architecture and the low-level requirements per software item belong. Writing requirements for those modules in this change would mean analysing them under a hazard register written the same day, which is the coupling the tooth ordering exists to avoid.

Round 2, on `39678fb` (with one later docs-only commit, `27d01d2`, adding fix
round 1's attestation to the plan): **PASS WITH FINDINGS, seven.** The
reviewer verified each of round 1's eleven dispositions against the tree —
**nine verified, two partially** — re-checked every citation in all four
documents rather than only the six it had flagged, and again ran every unit's
suite and all four gates itself, plus both clippy invocations.

The two partial verifications matter more than their count. Round 1's
disposition 2 claimed the register's RC-e2uvje evidence sentence had been
corrected; the reviewer's `git diff` showed that line untouched, so the fix
round reported work it had not done. Round 1's disposition 11 added the scope
paragraph it promised but rested it on a module list that is not exhaustive.
Both are dispositioned in fix round 2, and the first is why this record states
the review's verification of dispositions rather than the author's report of
them.

**finding-12**: The scope paragraph added for round-1 finding 11 rests on a module list that is presented as exhaustive and is not. `org-node/docs/risk/2026-09-09-org-node-hazards.md:16-19` says "Concretely that is every module of `org-node/src`" and then lists twelve; `chain.rs` (which defines the `ChainReader` trait and `MockChain` — the seam through which RC-6a2dke's decisive chain read arrives, and the type `ChainOpsReader` at `org-node/src/service.rs:1536-1540` implements), `error.rs` and `ids.rs` are absent. The claim at `:29-30` ("every module listed above was surveyed and its trust boundaries walked") therefore does not reach the whole unit, and the same twelve-item list is repeated as the basis of the requirements' scope statement at `org-node/docs/requirements/2026-09-09-verify-and-commit.md:139-146`. Under class C a scope overclaim in the paragraph that exists to make scope honest is the wrong place for it.
disposition: accepted. The three missing modules were added to both lists, and the resulting set was checked against the crate's actual file list so the claim is exhaustive rather than approximately so. The reviewer's last sentence is why this was not deferred: the paragraph's whole purpose is to state scope honestly.

**finding-13**: The register's structural evidence for RC-e2uvje is run by no gate, and the register does not say so. `org-node/docs/risk/2026-09-09-org-node-hazards.md:261-264` cites `org-node/src/lib.rs:2` and `Cargo.toml:22-25` as "the structural support for RC-e2uvje". No `verify_commands` entry runs clippy on org-node (`org-node/.guardrails/config.yaml:70-72` is `cargo test` plus `quint typecheck`) and no CI job does either (`.github/workflows/rust.yml:80-87` runs `cargo clippy` for org-members and on-chain-client only). Round 1's finding 2 is the proof: exposing `test_fixtures` broke `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]` and every gate stayed green. The same register refuses at `:556-560` to mint a control whose "evidence exists and counts for nothing", yet leaves this control's ungated half stated without qualification, and the residual-risk paragraph at `:284-294` names only the fuzzing depth and the blob bounds as shortfalls.
disposition: accepted, and the round's most consequential finding, because it turns the register's own standard against a claim the register makes. The passage now states the whole position: where the denial is declared, that the fixtures module is its one allow and never ships, that no `verify_commands` entry and no CI job runs clippy on this crate, and that round 1's regression is the proof a violation can land with every gate green. RC-e2uvje's residual-risk paragraph carries the shortfall alongside the fuzzing depth and the blob bounds, and a new not-minted control asks for clippy on org-node and app in CI and in `verify_commands`. The gap is also under Gaps below, because this change does not close it.

**finding-14**: PR-u4c2vp, minted in the fix round, was not propagated to any of the places that count or schedule problem reports. `org-node/.guardrails/config.yaml:29` still says "three open problem reports in doc_problems" where `check-trace.sh` reports `PR 4` / `open 4`. `docs/plans/2026-09-09-org-node-risk-analysis.md:106` lists only PR-hvg2dy, PR-vt244s and PR-2dmjzj as the file's new items, and `:47-48` still frames the survey's output as "The two defects: problem reports now, fixes later". `docs/plans/2026-09-05-ratchet-gap-analysis.md:165` says "PR-hvg2dy moved plus two new reports". `docs/plans/2026-09-05-ratchet-setup.md:153-165` carries a dated fix item for PR-vt244s, PR-2dmjzj and PR-hvg2dy but none for PR-u4c2vp, and its "Problem budget re-check" item states "org-node holds three open reports" — so the new report's 30-day age limit (2026-10-09) is the only one of the four with no owner's item behind it.
disposition: accepted, all five places corrected: the unit config's ledger comment, both bullets of this change's plan, the gap analysis's tooth-3 row, and the setup checklist, which now carries a dated fix item for PR-u4c2vp and a budget line reading four. The reviewer's last clause is why this was not cosmetic — a report with no owner's item behind it is a report whose age limit arrives unattended.

**finding-15**: The phrase that decides REQ-ztdza4's and RC-b6mydy's reach is not a defined term. REQ-ztdza4 (`org-node/docs/requirements/2026-09-09-verify-and-commit.md:101-107`) and RC-b6mydy (`org-node/docs/risk/2026-09-09-org-node-hazards.md:313-315`) are scoped to "the admission-and-update Receive operation", but `org-node/docs/CONTEXT.md:63-67` defines only *Receive operation* — "One acceptance of one Envelope by the node, from opening the connection to committing or rejecting" — which covers `receive_and_self_delete_if_revoked` as squarely as `receive_and_verify`. Neither glossary nor the requirements file maps "admission-and-update" onto an operation; a reader must infer it from the register's residual-risk prose at `:336-344`. In the same paragraph the Terms line (`:11-15`) names *Join request*, which no requirement below uses, and *Persona*, which appears only lower-cased at `:113`.
disposition: accepted. This is round 1's finding-1 disposition landing on an undefined phrase, which would have made the scoping unreadable against the glossary — the thing the scoping existed to fix. REQ-ztdza4 now names the operation by what it does, a message accepted through the receive operation that admits the software to an Organisation or updates its record, as distinct from the one that acts on the software's own removal; RC-b6mydy's second clause matches; and the glossary's **Receive operation** entry distinguishes the node's two receive operations, since the distinction now carries requirement weight. The Terms paragraph drops *Join request*, which no requirement uses, and *Persona* is capitalised where REQ-uxv2x2 uses it.

**finding-16**: REQ-mr5abb's rejection clause has no test. The requirement (`org-node/docs/requirements/2026-09-09-verify-and-commit.md:67-71`) says the mark advances "only when an Envelope has been committed, never on a rejection at any step". The two tests annotated to it cover forward-only advance (`org-node/tests/verify_against_chain.rs:169-177`) and that `check` does not mutate (`:244-253`), the second conceding in its own comment that it "documents the contract the type system enforces". No test drives a rejection at steps 4-8 of `verify_envelope_against_chain` (`org-node/src/verify.rs:62-82`) and then reads the mark, and no service-level test asserts `OrgRecord.last_seq` unchanged after a rejection — `update_relayed_by_a_non_member_after_admission_is_rejected` (`org-node/tests/admission_sender.rs:288-296`) asserts epoch, root and member count only.
disposition: accepted; the clause with the real teeth had the weakest evidence of the fifteen. `update_relayed_by_a_non_member_after_admission_is_rejected` now also carries REQ-mr5abb and asserts that B's persisted sequence mark is unchanged by the rejected relay, reddened by committing the record before the sender check; the root-mismatch test asserts that the mark handed in is unchanged after a rejection at the last step. The attestations are in the plan and in the red → green table above.

**finding-17**: The citation for where the production `ChainReader` is built points at a blank line and excludes the line it is about. `org-node/docs/risk/2026-09-09-org-node-hazards.md:159-161` states the reader "is built by `receive_and_verify` from the node's own `read_state` call ... (`org-node/src/service.rs:922-926`, `:1012-1013`, and the adapter at `:1532-1540`)". `service.rs:1011` is `let chain_reader = ChainOpsReader { state: chain_state };` — the construction; `:1012` is blank and `:1013` is the call into verification. Same class of defect as round-1 finding 5, in a range that round did not reach.
disposition: accepted, corrected to include the construction. Three rounds of citation findings in one document is a pattern worth naming rather than only fixing: the author's own check missed five, fix round 1 found a sixth, this is a seventh. What the register can now claim about its citations is the reviewer's enumeration, not the author's assurance.

**finding-18**: The ISO 14971 control-introduced-hazard entry mis-attributes its own rejection. The entry headed "**RC-m4r75s and RC-e5atck (strict ordering)**" (`org-node/docs/risk/2026-09-09-org-node-hazards.md:444-451`) then says the out-of-order envelope "is rejected at the base-root check" — which is org-members' REQ-4umsuz (step 5, `org-node/src/verify.rs:64-68`), neither of the two controls named in the heading. Traced through `verify.rs:46-87`, an out-of-order envelope carries a sequence number greater than the mark and so passes `SeqGuard::check`, and passes the epoch check; it fails at step 5. The hazard is real and correctly assessed, but it is not one these two controls introduce, and this clause is the one place the attribution has to hold.
disposition: accepted. The reviewer traced the path and is right: an out-of-order envelope passes both ordering checks and fails on org-members' base-root rule, so the entry did not belong under hazards these controls introduce. The hazard and its S3/P2 assessment stand, the rejecting check is correctly attributed, and the entry moved to the stale-view prose hazard where its consequence — a device left behind with no requeue and no resend — already lived. ISO 14971's requirement to examine each control for what it breaks is not served by crediting a control with a rejection some other rule performs.

Round 3, on `83b14e3`: **PASS WITH FINDINGS, nine.** All seven of round 2's
dispositions verified against the tree, two of them with a caveat that became
a finding of its own (the counts around finding-18's move, and a claim in the
paragraph it rewrote). The reviewer again ran every unit's suite, all four
gates and both clippy invocations itself, and checked every citation in the
four documents and the plan for the third time.

The pattern across three rounds is worth stating plainly, because it is what
a class C review is for: eighteen of the twenty-seven findings are precision
defects in prose the author wrote and believed to be accurate — citations off
by one or two lines, counts that stopped matching after an edit, a phrase
removed in one file and left standing in another, and claims about other
documents that were true when the survey was made and not when it was
written. Two rounds' worth of them were introduced by the fix rounds
themselves.

**finding-19**: This change writes org-node's hazard analysis, but org-node's two earlier dependency-expectation files — both in this unit's own regulated ledgers — still assert that it has none, and they are the stated reason REQ-ysyu9g and REQ-q92yac carry no `(implements: RC-…)`. `org-node/docs/requirements/2026-09-06-dependency-expectations.md:24-28`: "Neither implements a risk control of org-node's: org-node has no risk analysis yet (tooth 3 …). When that analysis is written, both items are candidates for `(implements: RC-…)`". `org-node/docs/risk/2026-09-06-dependency-expectations.md:8-9`: "because org-node has enumerated none of its own yet"; and `:56-61`: "org-node's register does that when it is written. Until then the two items above are candidates for `(implements: RC-…)` in that register." The new register does the opposite bookkeeping (`org-node/docs/risk/2026-09-09-org-node-hazards.md:100-109`, `:808-811`), so a reader of the SRS is told there is no register while the RMF holds six hazards and nine controls. These are prose paragraphs, not items, so the "edit items in the file that defines them" rule does not bar a dated correction.
disposition: accepted. Both files carry a dated 2026-09-09 correction naming the register, the hazard each expectation was written toward, and the reason the annotation is still withheld — an unmet expectation carrying `(implements: RC-…)` fails this unit's gate on every run, and the providers have until 2026-12-05. The items themselves are untouched. The reviewer is right that this is the ledger contradicting itself inside one unit, which is worse than a stale cross-reference.

**finding-20**: The change closes shortfalls that org-members' register still records as open, and leaves that register's residual-risk statement standing. `org-members/docs/risk/2026-09-02-membership-hazards.md:236-237` reads "Residual risk: reduced, **not acceptable**, because the decisive part of this control is neither stated as a requirement nor under any gate" — yet `org-node/docs/risk/2026-09-09-org-node-hazards.md:172-174` says in as many words that "the independence org-members' register asks its consumer to supply … is therefore structural here, and RC-6a2dke is the requirement it was missing", and REQ-wp2nyc/REQ-bvh8v6 now run under org-node's gate. The same file's bullet at `:247-250` ("`org-node` appears in no `verify_commands`, no `Makefile` target and no CI job, and is in neither `strict_paths` nor `test_paths`") was already false at `master` (tooth 1 gave org-node all four on 2026-09-05) and this change relocates exactly those seven tests into `org-node/tests` with `verifies:` annotations. Not-minted controls 3, 4 and 5 are all satisfied or largely satisfied and still listed as open. The change asserts "the register is otherwise unedited"; the provider-may-not-cite-a-consumer's-items rule permits a file reference, exactly as the 2026-09-09 correction two bullets down already does.
disposition: accepted, and this is the round's most substantive finding: a provider's register left claiming its consumer supplies nothing is a false statement about the system, not a stale cross-reference. Dated 2026-09-09 corrections were added at four places in org-members' register — the residual-risk statement, the "run by nothing" bullet, and not-minted controls 3, 4 and 5 — each naming org-node's two files rather than any of its items, as the cross-unit rule requires, and each saying which part remains open (control 3 still lacks a seed corpus and a libFuzzer lane; control 4 still lacks coverage, architecture and low-level requirements for org-node). The claim in org-node's register that org-members' file is "otherwise unedited" was corrected to list these four corrections, so the sentence matches the diff.

**finding-21**: The control-introduced-hazard accounting is short by one. `org-node/docs/risk/2026-09-09-org-node-hazards.md:473-477` says "Three of the nine controls have something to record" and lists RC-gfn6kr, RC-jjsz97, RC-b6mydy; `:651-654` says the introduced hazards "are two, both acceptable (S2/P1)". But `:794-802` (REQ-uxv2x2, realising RC-wqgm2p) states "Introduces a hazard the survey found and this analysis records without minting: … a spurious self-delete on an inconsistent store … S2/P1 on the state it needs; acceptable". That is a fourth control with something to record and a third introduced hazard, carrying a severity, a probability and an acceptability verdict, and it appears in neither the section nor the count. Round 2's fix sharpened the summary from "three hazards the controls introduced" to "are two", so the edit made the contradiction explicit.
disposition: accepted. The spurious self-delete is now the fourth bullet of the control-introduced section, with the presence test that causes it, its S2/P1 assessment, its acceptability under the matrix and the test worth adding when that code is next touched; the section reads four of nine controls and the residual summary reads three introduced hazards; and REQ-uxv2x2's derived assessment points at the bullet instead of repeating it. ISO 14971 asks what each control breaks, and an entry that exists in the derived assessment but not in the section that answers that question is answering it in the wrong place.

**finding-22**: Citation off by one. `org-node/docs/risk/2026-09-09-org-node-hazards.md:277-279` says test_fixtures "is compiled into the library under the `test-support` feature (`org-node/src/lib.rs:41-42`)". In the tree line 41 is the tail of a comment, line 42 is `#[cfg(any(test, feature = "test-support"))]`, and the module declaration `pub mod test_fixtures;` is line 43. The evidence for the claim is `:42-43`.
disposition: accepted, corrected to the two lines that carry the claim. Third round, third citation finding, and this one sits inside the passage round 2 added to make a different claim honest.

**finding-23**: The stale-view passage asserts a check that is never reached. `org-node/docs/risk/2026-09-09-org-node-hazards.md:534-535`: an out-of-order envelope "passes RC-e5atck's epoch check (the chain is at the later epoch, which is newer than the last committed). It fails at step 5 of `verify.rs`". The epoch check is step 7, at `org-node/src/verify.rs:76-78`, after the base-root return at `:66-68`; verification returns `DeltaBaseMismatch` before the chain is read at all, so RC-e5atck's check is never evaluated. The conclusion (neither ordering control is what rejects it) is right; the passage states as fact something only counterfactually true, in the one paragraph round 2 asked to be made precise.
disposition: accepted. The passage now says what the code does: such an envelope passes the sequence check and is rejected at the base-root comparison, before the chain is read at all, so neither ordering control performs the rejection. The conclusion and the S3/P2 assessment stand. The reviewer's distinction — asserting as fact something that is only counterfactually true — is the same class of defect as crediting a control with another rule's rejection, which is what round 2 asked to be fixed in this very paragraph.

**finding-24**: The phrase finding-15 removed survives in the problems ledger. `org-node/docs/problems/2026-09-09-org-node-problems.md:180-182`: "It is also the reason RC-b6mydy's second clause is scoped to the admission-and-update receive operation rather than to every message the node acts on." RC-b6mydy, REQ-ztdza4 and the register's own `:373-375` were all reworded away from that undefined construction; this one was not, and CONTEXT.md's glossary defines *Receive operation*, not "the admission-and-update receive operation".
disposition: accepted, reworded the way the register and the requirement were, and the tree was grepped for the phrase to confirm none remains. A term removed from three places and left in a fourth is the shape of defect a whole-tree grep catches and a per-file edit does not.

**finding-25**: Glossary list inconsistency in the requirements file. `org-node/docs/requirements/2026-09-09-verify-and-commit.md:11-15` lists *Finalised block* as a root-glossary term the file uses — no requirement or paragraph in the file uses it — while omitting *Membership record*, which REQ-ztdza4 uses at `:106` and which `org-node/docs/CONTEXT.md:14` names as a root-level cross-unit term. Fix round 2 edited this list (dropping *Join request*, capitalising *Persona*) without reconciling the other two.
disposition: accepted, both corrected, and the whole list audited in both directions — every term it names is used in the file, and every cross-unit term the file uses is named. The reviewer's observation that fix round 2 edited this very list without auditing it is the reason the audit was made exhaustive rather than incremental.

**finding-26**: The unit manifest's dependency commentary is stale after this change. `.guardrails/units.yaml:40-42`: "org-node has two requirements, both `expects:` items, and an expectation is not an export; on-chain-client and app have none. So all three export surfaces are empty". org-node now has seventeen requirements. The conclusion about export surfaces still holds; the count does not, and the manifest is where a reader goes for the cross-unit picture.
disposition: accepted, corrected to seventeen — fifteen ordinary requirements and the two expectations — naming the file that added the fifteen, with the conclusion intact and re-verified: `check-units.sh --exports org-node` still prints nothing, because none of the seventeen carries `exported: yes`.

**finding-27**: Two inaccuracies in the plan's survey note, plus one comment falsified by the source change. `docs/plans/2026-09-09-org-node-risk-analysis.md:74-76` says "Two of the bullets below are the survey as it was made and are wrong for those reasons", but only one bullet carries a corrected citation — the `org_pub_key` bullet. The `ChainOpsReader` bullet cites the adapter at `:1532-1540`, which is correct and was never edited; the `:1012-1013` error existed only in the register. Separately `:157-158` says "`envelope.rs` and `service.rs` keep their unit tests; they still use `crate::test_fixtures`" — only `org-node/src/envelope.rs:96` does. And `org-node/tests/transport_networked.rs:28` still carries "Inline genesis_and_admit (test_fixtures is lib-private)", which this change's `pub mod` under `test-support` makes untrue for that target.
disposition: accepted, all three corrected. The third is the one worth noting: a source comment made false by this change's own visibility edit, in a test file the change otherwise only annotated. Only the comment was touched; no test code or annotation changed.

Round 4, on `639bd93`: **PASS WITH FINDINGS, eleven.** All nine of round 3's
dispositions verified or partly verified, the author's two direct corrections
checked, every citation and every count in the register re-derived, and the
whole suite plus both clippy lines run again by the reviewer.

**What the four rounds show, and the decision it forced.** Thirty-eight
findings were raised across four rounds: eleven, seven, nine, eleven. Nothing
in the code was ever wrong — the source diff was confirmed behaviour-neutral
in every round — and no severity, probability or acceptability verdict was
overturned. What the reviews found, again and again, were defects in the
change's own prose: citations off by a line or two, counts that stopped
matching after an edit, a phrase removed in three places and left in a
fourth, and claims about other documents that were true when the survey was
made and false when the register was written. Rounds 3 and 4 found that a
growing share of these were introduced *by the fix rounds themselves*, in the
sentences those rounds added to make earlier findings honest — a tally of
edits, an "except that", an "unchanged", a list asserted to be exhaustive.

So fix round 4 did two different things. It fixed the substance, and it
**removed the bookkeeping instead of correcting it**: where a sentence's only
content was a count of this change's own diff, the count was deleted rather
than recomputed, keeping the fact a reader needs (org-members' register
carries dated corrections; here is what they say and what remains open) and
dropping the tally that cannot be maintained. That is the author's judgement
about the cause, not the reviewer's finding, and it is recorded here because
the next per-unit risk analysis — on-chain-client's, then the app's — should
not repeat the pattern.

**finding-28**: The count of edits to org-members' register is still wrong, and the sentence stating it is internally inconsistent in three further ways. `org-node/docs/risk/2026-09-09-org-node-hazards.md:32-41` says "edited in six places … one line that named a problem report … four passages … and a marker in its residual-risk table". The diff edits seven distinct places, confirmed by the seven `2026-09-09` correction sites. The enumerated "four passages" itself lists five. Line 33 claims "all of them dated corrections that leave the original wording standing", which is false for the PR-hvg2dy bullet, where the ID was deleted and replaced by "a problem report". And the two places describing the table marker contradict each other: one says its row summarised the shortfall of "the first" of those four, the other "the fourth".
disposition: accepted, and answered by deletion rather than by a corrected count. Every tally of this change's own edits was removed from the register and from the plan, and replaced by the fact a reader needs: org-members' register keeps its wording and carries dated 2026-09-09 corrections wherever this change makes it out of date, one of which replaces a problem-report ID with the file that now holds it, and no item in it is altered. The blanket "leave the original wording standing" is gone, since that one word was replaced. A count of one's own diff is a claim that goes stale on the next edit, and this is the second round it went stale in.

**finding-29**: Four further passages of org-members' register are now false and carry no marker, while exactly parallel passages did get one. (a) `:377-381`, "the target exists, sits outside `test_paths`, carries no `verifies:` annotation, has an empty seed corpus, and is run by no lane"; three of those four clauses are false. (b) `:428-432`, HAZ-h58jn6's residual, "implemented, tested in-tree, run by no gate". (c) residual table `:533`, "the target that would support it is unrun". (d) residual table `:535`, "closing control ungated and unrun". The author marked only row `:531`, so the table reads as if one row were affected and three were not.
disposition: accepted; all four now carry dated 2026-09-09 corrections in the shape the others use, the two table rows with the marker style the first row already had. The reviewer's point about parallelism is the substance of it: marking one row and leaving three identical ones makes the register look checked when it is not.

**finding-30**: The register's defect accounting is short by one, in two places. `…hazards.md:118-125` says "Two hazards are the shape of a defect … Both are filed as problem reports"; three are filed (PR-vt244s, PR-2dmjzj, PR-u4c2vp), and three not-minted controls are their fixes. `:692-693` repeats it: "two defects on the publish and admission paths", omitting the revocation-receive-path defect.
disposition: accepted, both corrected to three with the third named, and the three not-minted controls that are their fixes named alongside. Same shape as round 3's finding-21: a report filed in a fix round and not carried into the prose that counts them.

**finding-31**: A correction added to org-members' register states a falsehood about the fuzz targets. `:611-613`: "the target is driven as an ordinary `cargo test` over a fixed set of inputs, not fuzzed". Measured in this review: `cargo test --test fuzz_envelope_decode` reports 195,826 rng inputs in one second under bolero's generative engine. The same file at `:362` and org-node's register both say so correctly. The empty-corpus half is right.
disposition: accepted, and this is the finding that matters most in the round, because a correction written to make a provider's register accurate made it inaccurate instead — and in the direction that understates the evidence, which is the direction a register must never err in. It now says the target runs under bolero's generative engine for its one-second budget at every merge, and that what remains absent is the seed corpus and a libFuzzer lane.

**finding-32**: `chain_read.rs` is surveyed but is in neither enumeration of modules without a requirement (`…hazards.md:45-51`, `…verify-and-commit.md:147-151`), and both are written as exhaustive. It is the one module of the fifteen that is neither given a requirement nor declared to be without one — and it is the module PR-hvg2dy is filed against.
disposition: accepted, added to both enumerations with that last fact named, and both lists re-checked against the surveyed set and the fifteen requirements.

**finding-33**: The load-bearing noun of three items is a word the cited glossary puts on an *Avoid* list, and the object that actually carries the Organisation secret has no term. `org-node/docs/CONTEXT.md:18-22` defines **Envelope** with "_Avoid_: message"; RC-b6mydy, REQ-xa6smf and REQ-ztdza4 all turn on "message" — and must, because what arrives is a `WireMessage` (`org-node/src/transport/wire.rs:16-21`) carrying the Envelope plus `org_secret` and `genesis_snapshot`, and HAZ-ep6uzs depends on exactly that difference. *Receive operation* is defined as "one acceptance of one Envelope", narrower than what the requirements scope themselves to.
disposition: accepted, and answered by giving the wider thing a term rather than by bending the items to *Envelope*. **Wire message** is now defined in the unit glossary — what one device sends another in one receive operation: an Envelope, and on admission the Organisation secret and the snapshot the receiver needs — with *message* on its own avoid list; the three items say *Wire message*; *Receive operation* is widened to match; and the requirements' Terms list names it. No requirement's meaning changed, only its noun. The reviewer found the one place where the glossary and the requirements genuinely disagreed about what the software receives.

**finding-34**: `org-node/tests/verify_against_chain.rs:3-7` claims the first nine tests were relocated "unchanged", and the plan repeats "bodies unchanged". `rejects_root_mismatch_when_chain_root_differs` gained two assertions on the sequence mark and two further `verifies:` IDs in later rounds.
disposition: accepted; the word "unchanged" is gone from both. They now say the tests were relocated from those modules and annotated, with the additions the fix rounds made recorded in the plan. An "unchanged" that a later round changes is a claim with a short life.

**finding-35**: `org-node/docs/problems/2026-09-09-org-node-problems.md:62-65` says the text is the 2026-09-02 original "except that" its last paragraph named another report by ID. There is a second change: the ledger path was repointed when the ledgers moved. The register's "with its original text and opened date" does not account for it either.
disposition: accepted; both now say the body is the 2026-09-02 text with two mechanical changes — the report ID replaced by the file that holds it, and the ledger path updated — and that no statement of the defect was rewritten. Another exhaustive claim retired rather than re-enumerated.

**finding-36**: `org-node/.guardrails/config.yaml:63-69` leaves "Measured 2026-09-05 …: 38 passed, 0 failed" as the only figure for a `verify_commands` line this change replaces, adding only "The `--lib` count drops by the tests that moved" with no number.
disposition: accepted. The 2026-09-05 figure stays as the dated historical measurement it is, and the new line carries its own dated 2026-09-09 measurement, verified by running it: 50 passed, 0 failed, broken down per target, plus the two bolero targets that report no counts. A config comment is where the next reader looks first, and a stale figure there outlives every plan.

**finding-37**: The plan states the impact set from "The change touches org-node and org-members (one line of its register, one problems file)". It edits seven places in that register, and the file list repeats the understatement.
disposition: accepted, and answered the same way as finding-28: the plan now says which files the change touches without counting lines in them.

**finding-38**: Citation off by a step. `…hazards.md:551-554` says "the chain is not read until step 7 (`:76-78`)"; in `org-node/src/verify.rs` the read is `:72-75` and `:76-78` is the epoch comparison. The conclusion is right, the cited range names the check rather than the read.
disposition: accepted, corrected to the lines that carry the read. Fourth round, fourth citation finding, and the shortest-lived class of defect in the whole change: every one was a sentence whose argument was sound and whose pointer was off by a line or two.

Round 5, on `afc11b5`: **PASS WITH FINDINGS, ten** — and unlike rounds 3 and
4, two of them are about the software's safety rather than the change's prose.
The reviewer was asked to rank each finding by whether it would mislead a
reader about safety, evidence or open work, and it did: two mislead about
safety, two about the evidence and the change's reach, one about where open
work is recorded, one about the suite, and four are precision matters a reader
would not be harmed by. All ten were dispositioned in fix round 5.

The two safety findings are the ones this record should be read for, because
both are the analysis getting its own arithmetic wrong in the direction that
flatters the software:

- The 1 MiB frame bound's introduced hazard was assessed S2/P1 and
  **acceptable, "no control needed"**, on the reasoning that an Organisation
  too large to fit is "refused at admission, visibly". The reviewer traced the
  code: the send happens *after* the chain write on the admission path and the
  revocation path alike, so a snapshot over the bound fails inside the
  publish-before-persist window this same register assesses at S3/P2 and files
  as PR-vt244s. Re-assessed S3/P2 and not acceptable, with the snapshot bound
  added to the not-minted controls.
- Two identical harms carried different severities. A forgotten passphrase
  destroys a device's whole store and was assessed S2/P1 acceptable; a
  crash-truncated store file destroys the same thing and was assessed S3/P1
  not acceptable. The register's own method and the class C ADR put
  unavailability at S3 at its worst credible outcome, and the S2 leaned on a
  cross-reference to org-members' device-bound lock-out, which is a far
  narrower harm. Re-assessed S3/P1 and not acceptable.

After both corrections exactly one control-introduced hazard remains
acceptable, and the register's counts and summaries were reconciled to say so.

The evidence findings matter for the merge itself. The impact set the plan
stated — org-members, org-node, app — was correct when the plan was written
and became wrong when a later fix round edited `.guardrails/units.yaml`: a
change under the root `.guardrails/` directory maps to every unit, so
`check-units.sh --impact` now prints all four as touched. The reviewer ran
on-chain-client's suite itself rather than leaving the omission theoretical
(23 passed, plus three fuzz targets). The corrected set is what the gate table
above covers.

**finding-39**: RC-gfn6kr's introduced hazard is assessed only on the admission path, and the assessment that makes it acceptable is incomplete. `org-node/docs/risk/2026-09-09-org-node-hazards.md:495-501` says the 1 MiB bound "bounds the Organisation an admission can be delivered for. Assessed S2 / P1 — a large Organisation is refused at admission, visibly … Acceptable; no control needed", citing `service.rs:826-833`. But `revoke_member` also puts the whole pre-revocation record into the Wire message (`org-node/src/service.rs:1213-1214`), and `OrgEndpoint::send` runs it through `encode_frame` (`org-node/src/transport/endpoint.rs:215`, check at `org-node/src/transport/wire.rs:26-28`). So for a sufficiently large Organisation the bound refuses the *revocation* send at `service.rs:1224`/`:1234` — after the chain write at `:1172` and before the record update at `:1254-1261`, i.e. exactly inside the publish-before-persist window the same register assesses as **S3/P2** and files as PR-vt244s. The failure is then neither visible-at-admission nor recoverable: the chain carries a removal no device is told of and the administrator can no longer act. The receiver never even reads that snapshot (`receive_and_self_delete_if_revoked` builds its trie from its own store, `service.rs:1299-1310`), so the frame budget is spent on dead weight on the one path where exhausting it is unsafe. This would mislead a reader about safety: an introduced hazard is declared acceptable with "no control needed" on a pathway analysis that omits its worst case.
disposition: accepted in full, and the reviewer's trace is right about more than it claims — the admission path has the same ordering, so "refused at admission, visibly" was wrong on both paths, not just the revocation one. The entry is re-assessed S3/P2 and **not acceptable**, stating that the send fails after the chain has moved on either path, that this is the publish-before-persist window PR-vt244s covers, and that on the revocation path the snapshot is dead weight because the receiver rebuilds from its own store. Its controls are PR-vt244s's ordering fix and a new not-minted control, bound or omit the membership snapshot on the send path. Every count and summary that depended on the old assessment was reconciled.

**finding-40**: two identical harms carry different severities and therefore opposite acceptability verdicts, with no distinction argued. `…hazards.md:502-506` (RC-jjsz97's introduced hazard): "A forgotten passphrase is a lost store: the member's keys and record are gone and the member must be re-admitted with new keys. Assessed S2 / P1 … Acceptable; no control needed." `:637-644` (the keys-lost-on-crash prose hazard): a truncated store file loses "every persona's keys and every Organisation's record on that device. … Harm: unavailability until re-admission. Assessed **S3 / P1**", not acceptable, control not minted. Same loss, same recovery, and if anything the passphrase case is the harsher of the two (re-admission *with new keys*). The register's own "Method" section (`:84-92`) and `docs/adr/2026-09-01-safety-class-c.md:50` put unavailability at S3 at its worst credible outcome, so the S2 needs an argument and gets only a cross-reference to org-members' device-bound lock-out (`org-members/docs/risk/2026-09-02-membership-hazards.md:477-482`), which is a much narrower harm (one member cannot enrol a fifth device). This would mislead a reader about safety: it tells them a lock-out that destroys a device's whole store needs no control.
disposition: accepted. Re-assessed S3/P1 and **not acceptable**, argued in the entry: it is the same loss and the same recovery as the crash case, and strictly worse, because recovery is re-admission with new keys. Its control is not minted and is named with the atomic-write control it belongs beside, plus a documented re-enrolment procedure. The reviewer's point about the cross-reference is the substance of it — a member at the device bound still holds the devices they have, which is not the same harm at all, and borrowing that S2 was the error.

**finding-41**: the plan states an impact set the tool contradicts, and the unit it omits is the one whose gate would then not run. `docs/plans/2026-09-09-org-node-risk-analysis.md:15-19` and `:470-472` name three units. `check-units.sh --impact master..HEAD` prints **all four units as `touched`**, including on-chain-client, because the change edits `.guardrails/units.yaml` and `.guardrails/scripts/check-units.sh:186-192` sets `touched="$units"` for any `.guardrails/*` path. So the plan names three units where the gate demands four, and mislabels app. This would mislead a reader about the evidence: on-chain-client's `verify_commands` would be skipped. I ran them myself — 23 passed, 0 failed plus three bolero targets — so nothing is hidden today, but the plan's set is wrong.
disposition: accepted; both passages now say four units, all touched, with the reason (the root `.guardrails/` edit maps to every unit) and the history (the set was three when the plan was written and became four when a later round edited the manifest). on-chain-client's suite was run and its counts recorded, and the gate table above covers all four units. The reviewer running the omitted unit's suite rather than only naming the omission is what kept this a bookkeeping fix instead of missing evidence.

**finding-42**: the plan's "Files this change touches" list omits four files the change edits, and no task section names them: `.guardrails/units.yaml`, `org-node/docs/requirements/2026-09-06-dependency-expectations.md`, `org-node/docs/risk/2026-09-06-dependency-expectations.md`, and `org-node/tests/transport_networked.rs`. Two of the four are regulated ledger documents of the unit under analysis. This would mislead a reader about the change's reach: an auditor reconciling the diff against the plan finds four unexplained files, one of which is the cause of finding-41.
disposition: accepted; all four added with what changed and which round changed them, and the whole list then reconciled against `git diff --stat` in both directions. The four accumulated across fix rounds, which is exactly when a file list stops matching a diff.

**finding-43**: a stale citation in a ledger file this same change edited. `org-node/docs/requirements/2026-09-06-dependency-expectations.md:66-68` still says the contradiction is "filed as an open problem report in `org-members/docs/problems/2026-09-02-chain-reader-finality-doc.md`" — but this change empties that file, which now forwards to org-node's ledger. The change added a dated correction to this very file without touching the pointer.
disposition: accepted; repointed inside the dated correction the file already carries rather than as a silent edit, so the history stays readable.

**finding-44**: a surviving count that is wrong. The plan says "plus org-members' two and app's two, unchanged". org-members has **three** `verify_commands` entries; app has two.
disposition: accepted, and restated without a per-unit tally: the other units' commands are unchanged by this change and the gate runs each unit's own entries. The last surviving count of that kind.

**finding-45**: the glossary term minted for finding-33 is inaccurate about what leaves the sender. `org-node/docs/CONTEXT.md:24-29` defines *Wire message* as carrying the Organisation secret and the snapshot "on admission"; `revoke_member` also sets a snapshot (`service.rs:1213-1214`). The receiver ignores it there, so no hazard follows from the substitution — but the definition is what finding-39 relies on being complete, and it is not.
disposition: accepted; the definition is widened to be true of both paths, keeps the note that the secret and the snapshot lie outside the Envelope's signature, and says the receiver of a revocation does not read the snapshot — which is also what makes finding-39's "dead weight" statement checkable.

**finding-46**: the reason given for keeping the emptied file is not supported. `org-members/docs/problems/2026-09-02-chain-reader-finality-doc.md:13-15` says it is kept partly so "the 2026-09-02 verification record's reference to it still resolves"; that record names the report by ID and never the file.
disposition: accepted; that half of the reason is dropped and the chronology half kept, which the reviewer confirms is sound.

**finding-47**: a cited span does not contain what it is cited for. `…hazards.md:332-334` cites `service.rs:897-926` for "a chain read, a record rebuild and a signature check per connection"; the read is `:922-926`, the rebuild `:933-969`, and the signature check is inside `verify_envelope_against_chain` at `:1013`. The claim is true; the citation under-covers it.
disposition: accepted, corrected to cover what the sentence claims. Fifth round, fifth citation finding.

**finding-48**: a cluster of wording slips, none of which would harm a reader. (a) RC-b6mydy writes "receive operation" in lower case where the glossary term is *Receive operation*. (b) `org-node/.guardrails/config.yaml:65-69` groups `wire_frame_bound` and `store_at_rest` under "the verify-path and sequence tests relocated"; those two came from `transport/wire.rs` and `store.rs`. (c) `org-members/…:645-646` says "fifteen of them, each realising one of nine controls" — REQ-nhe2zu realises three. (d) PR-vt244s carries `affects: RC-wqgm2p` and neither the report nor the register says why a publish-path defect is filed against the self-delete control.
disposition: all four accepted and corrected; (d) gains the sentence that was missing rather than a changed `affects:` — an undelivered revocation is RC-wqgm2p never firing on the device being removed, which is why the defect degrades that control.

Round 6, on `96e1c36`: **PASS WITH FINDINGS, eight — verdict "not ready to
merge".** The reviewer's first task was to check fix round 5's two
re-assessments, and it verified both as sound in substance with every code
claim behind them exact. It then found that the *third* introduced hazard, the
one the register still called acceptable, was closed on the same reasoning the
register had just rejected twice.

That is finding-49, and it is the most consequential finding of the whole
change. The spurious self-delete was assessed S2/P1, acceptable, **no control
needed** — the only place in the register where a hazard was closed with no
control at all. Its severity rested on recoverability by re-admission, which
`docs/adr/2026-09-01-safety-class-c.md` withdrew on 2026-09-02 in as many
words (severity is a property of the harm, and an organisational fallback does
not make an injury less severe), and which the register itself had invoked two
bullets earlier to raise the passphrase lock-out to S3. The register's Method
section also asserts that every hazard in the file is S3, and this entry was
its only counter-example. Re-assessed S3/P1 and not acceptable, with a real
not-minted control in place of "a test worth adding when the code is next
touched" — which, as the reviewer says, is not a risk control.

The consequence is worth stating plainly, because it is the change's
conclusion: **after this round no assessed risk in org-node's register is
acceptable.** The unit's overall residual risk was already UNACCEPTABLE
against the intended use; what round 6 removed was the single exception the
per-hazard table still carried.

Three further findings would have misled a reader about evidence or open work:
an expectation described as the provider-side half of a control it is not
(REQ-q92yac closes an uncontrolled residual of HAZ-vxabf9, while RC-wqgm2p is
implemented and tested), the reach of the chain-reader defect overstated to
include the app's preflight and "the chopsticks tests" when the code shows one
test and no production path, and a comparative claim about recovery that was
wrong in both directions. Four were precision matters, including one word —
"unauthenticated" where the transport does authenticate the peer's device key
and only the record's authorisation is missing.

**finding-49**: The one hazard this register still calls acceptable is assessed by reasoning the register itself rejects two bullets earlier, and the whole "exactly one acceptable risk" conclusion rests on it. RC-wqgm2p's introduced hazard — the spurious self-delete — reads "The harm is unavailability, recoverable by re-admission. Assessed **S2 / P1**: it needs a store state the admission path does not produce" (`org-node/docs/risk/2026-09-09-org-node-hazards.md:558-566`). Two things are wrong with that. First, the sentence offered as the severity justification is a *probability* argument, and the only severity argument present is recoverability by re-admission — which is exactly the move the register forbids in the bullet immediately above, where the passphrase lock-out is raised to S3 because unavailability is S3 at its worst credible outcome (`:539-541`), and which the class C ADR itself withdrew on 2026-09-02: "severity is a property of the harm and an organisational fallback does not make an injury less severe" (`docs/adr/2026-09-01-safety-class-c.md:51-52`). A spurious self-delete deletes the node's Organisation record and marks the persona `Revoked` (`org-node/src/service.rs:1369-1375`) — the member cannot reach the material at the moment a decision needs it, which is the S3 unavailability pathway verbatim. Second, the Method section asserts "every hazard below is S3" (`:94-95`); this entry is the single counter-example and the Method section is not qualified for it. If it is S3, the matrix makes it UNACCEPTABLE at P1, the "no control needed" disposition falls (a "test worth adding when the code is next touched" is not a risk control), and the register's headline count, the introduced-hazard intro and the residual summary all become "none". This misleads a reader about the software's safety and about what work remains open: it is the only place in a class C register where a hazard is closed with no control, and it is closed on grounds the same document rules out.
disposition: accepted in full, and it is the finding this record should be read for. Re-assessed **S3/P1, not acceptable**, with the severity argued from the harm and the probability from the store state the branch needs. Its control is now a numbered not-minted control — make the presence test independent of a Persona's recorded Organisation, with a test — cross-referenced from the entry. Every count and headline that rested on the old assessment was reconciled, and the result is that no assessed risk in this register is acceptable, which makes the Method section's claim true without qualification. An ID was minted in case the control belonged in the register as a real RC; it was deliberately left unused, because the control is neither implemented nor tested in this change and minting it would assert a traced control where there is none — the same rule the register applies throughout.

**finding-50**: The register and the problems ledger both name the app's preflight as a consumer of the reader whose doc-comment is the open defect, and it is not one. The register says of `OnChainReader` that it is "the cached, refresh-on-demand adapter the chopsticks tests and **the app's preflight** use" (`…hazards.md:212-214`), and PR-hvg2dy's move note repeats it (`org-node/docs/problems/2026-09-09-org-node-problems.md:74-75`). `org-node/src/preflight.rs:114` calls `client.get_org_state(admin, None)` on `OrgRegistryClient` directly and documents it correctly as "Reads at the latest finalised block" (`:107`); it never constructs or touches `OnChainReader`. A repo-wide search outside `org-node/src` returns exactly one consumer, `org-node/tests/chain_genesis_e2e.rs` — one chopsticks test, not the plural. So the reader with the wrong doc-comment is reached by no production path and no gated path at all. This misleads a reader about the evidence bounding an open problem report: the sentence exists precisely to say how far the defect reaches, and it names a reach the code does not have.
disposition: accepted; both sentences corrected to the reach the code has. The correction makes the defect's blast radius smaller, not larger, so it is stated plainly as such, and the report stays open on the grounds it already gives: the doc-comment of a security-relevant control is wrong, and a second reader written from it would implement the weaker read.

**finding-51**: REQ-q92yac is presented as the provider-side half of a minted control, and it is the provider-side half of no control — it closes a residual the control does not address. The register states "Each is the provider-side half of a control below" of both expectations (`…hazards.md:113-114`), and the conclusion counts "**two controls** whose decisive half is an open expectation" (`:757-758`). That holds for REQ-ysyu9g, whose subject is RC-6a2dke's decisive input. It does not hold for REQ-q92yac: RC-wqgm2p is fully implemented by REQ-uxv2x2 and tested, while REQ-q92yac asks org-members to relate a leaf's device set to its Member-as-a-group key, which the register itself files as the third of HAZ-vxabf9's residual counts (`:450-454`) and which the derived-assessment section frames correctly as a hazard, not a control (`:914-917`). This misleads a reader about what work remains open: it says a minted, implemented, tested control is half-delivered pending a provider, when what is pending is a separate uncontrolled residual of the same hazard.
disposition: accepted; both passages corrected. One control has its decisive half open on a provider; the other expectation closes an uncontrolled residual of the removed-device hazard. Every sentence that paired the two expectations symmetrically was checked for the same inherited error. The reviewer's distinction is the right one and it matters for the owner's checklist: what org-members owes is not the completion of a control but the closing of a residual.

**finding-52**: The comparative that carries RC-jjsz97's re-assessment is contradicted by the register's own text about the hazard it compares against. The passphrase bullet argues S3/P1 partly because "the recovery is **strictly worse** … rather than a restored file" (`…hazards.md:541-546`). The crash hazard's recovery, as this register states it, is the same one — "unavailability until re-admission" (`:683-690`). There is no restored file in either case, no backup exists anywhere in the design, and the control the crash hazard names is a prevention, not a restore. If anything the passphrase case is the *more* recoverable, since the ciphertext is intact. The S3/P1 conclusion survives without the clause, so this misleads about the evidence behind the re-assessment and, read from the other end, about what a crash leaves recoverable.
disposition: accepted; the clause is deleted. The conclusion stands a fortiori on "the same loss, of the same file, as a hazard this register assesses S3/P1", and nothing more is claimed. A supporting argument that is false in both directions is worse than no supporting argument, because it invites the reader to check it and rewards them with a contradiction.

**finding-53**: RC-gfn6kr's introduced hazard is given a probability it borrows rather than estimates, and the register records no figure for the only thing that triggers it. The bullet says "the severity is that hazard's, and so is the probability, because the frame bound does not create the window" (`…hazards.md:520-524`); the parent's P2 is justified as "a send failure is an ordinary event". This pathway instead requires the snapshot to exceed 1 MiB. From the `MemberSnapshot` encoding and org-members' field limits the bound bites at roughly 1,800 members at worst and on the order of 9,000 typically — real for a government deployment, and unbounded because org-members caps no member count, but not an ordinary event, and unlike the other triggers it is deterministic and permanent once crossed. A precision matter about the evidence for one probability estimate; it cannot mislead about the verdict, because S3 is unacceptable at every probability and both controls are recorded.
disposition: accepted. The entry now states the order of magnitude at which the bound bites, how it follows from the snapshot encoding and org-members' limits, that no member cap exists, and that this trigger is deterministic rather than intermittent — and argues the probability on those terms rather than borrowing it. The reviewer also re-derived every ordering and citation claim in the bullet and found them exact, which is why this is about the number and not the verdict.

**finding-54**: The frame-bound entry simultaneously counts a distinct introduced hazard and identifies it with a hazard already counted, and the passphrase entry does the same, while the register applies the opposite test to its prose list — excluding two entries from its eleven precisely because they restate hazards already counted (`:741-744`). The register never says whether the two re-assessed introduced hazards are new hazards or new triggers for hazards inside the eleven, so a reader cannot tell how many distinct hazards the file asserts. A precision matter in the register's own accounting, which is what the counts exist to make checkable.
disposition: accepted; the register now states which test it applies and applies it consistently, so the arithmetic is checkable by a reader rather than by its author. The reviewer's verification of every other count in the file — six hazard definitions, nine controls, six table rows, seven prose entries, sixteen not-minted controls, fifteen requirements with fifteen derived assessments, four problem reports, the eight places org-members' register names org-node, the seventeen requirements in the manifest — is what makes this the last of the counting findings rather than one of many.

**finding-55**: The register credits the review round with performing a risk assessment: "Both of the two were re-assessed on 2026-09-10, **in review round 5**" (`…hazards.md:494-499`), and the residual summary repeats it (`:731-732`). Review round 5 found the defects; the fix round re-assessed them. The file's own convention elsewhere is the correct one. A wording and documentation-discipline matter, but the wrong one to be loose about: under this project's independent-review discipline the reviewer must not be the author of the assessment, and this sentence records that they were.
disposition: accepted, and the reviewer is right that this is the wrong sentence to be careless in. Reworded to the file's own convention: what the earlier assessment was, that review round 5 found it defective, and that it was re-assessed in the fix round that followed. Independence is the property the whole review discipline rests on, and a register that misdescribes who assessed what undermines it on paper even where the practice was correct.

**finding-56**: A residual-risk paragraph calls the transport peer "unauthenticated", which is the opposite of the property the register establishes elsewhere (`…hazards.md:331-335`). The QUIC handshake does authenticate the peer's ed25519 device key — the register's own P2 justification for the sender's-word hazard cites the transport module for exactly that, and `recv_one` returns the authenticated remote device key (`org-node/src/transport/endpoint.rs:245-248`). The word wanted is *unauthorised*, or "a stranger the record does not name".
disposition: accepted, corrected, and the rest of the change checked for the same slip. The substance of the sentence — that no rate limit, allowlist or read timeout guards the accept boundary, so a stranger obtains a chain read, a record rebuild and a signature check per connection — is unaffected and the reviewer verified its split citation.

Round 7, on `2e00279`: **PASS WITH FINDINGS, thirteen — and none of them a
safety misstatement.** This round was dispatched with the merge bar stated in
the prompt, so the reviewer could rank against it: the author would merge if
nothing found would mislead a reader about the software's safety, the evidence
behind a claim, or what work remains open, and findings below that bar would
be recorded with a disposition rather than triggering another rewrite. The
reviewer was told plainly not to soften a real finding to help the change
merge and not to inflate a wording matter to block it.

Its verdict: nothing reaches the bar as a safety misstatement. Five findings
sit above it — four on the evidence behind a claim, one on open work — and all
five are defects in documents and records, not in the code, the requirements
or the test evidence. Eight are below the bar.

**All five above the bar were fixed** in a final, deliberately narrow fix
round; the eight below are dispositioned here and not fixed, for the reason
rounds 3 to 6 established: rewriting has itself been the largest source of new
prose defects in this change, and eight wording matters are cheaper to record
accurately than to re-edit and re-review.

The round also re-derived, independently, the two things the previous round
had changed: the arithmetic behind the frame bound's threshold (583 bytes per
member snapshot from the field limits, so about 1,800 members at maximum
lengths and about 8,000 typically against a 1 MiB frame) and every count in
the register (six identified hazards, five distinct prose entries, one
distinct introduced hazard, twelve in all with two further triggers, nine
controls, fifteen requirements, seventeen not-minted controls, six table
rows). Both check out. And it read the new abnormal-input test line by line
and confirmed it discriminates: with the signature check removed the envelope
fails instead at the epoch check, which reddens the error-kind assertion.

**finding-57**: `docs/plans/2026-09-05-ratchet-setup.md:197-201` — the "restriction on use" checklist item, the project's own list of open safety work, says org-node's register reaches UNACCEPTABLE "including **two of the three** hazards its own controls introduce, re-assessed in **review round 5** as unacceptable in their own right." Both halves are now false. The register says all three are unacceptable (`org-node/docs/risk/2026-09-09-org-node-hazards.md:513`, `:830`), and it explicitly withdrew crediting the reviewer with the re-assessments (`:517-519`), which was round-6 finding 55. Fix round 6 reconciled the register's counts and headlines but not this one, so the sentence understates the register's conclusion and reproduces exactly the attribution defect finding 55 corrected. [ranked: evidence, arguably safety, since a reader of the checklist would take the introduced-hazard picture to be better than the register says]
disposition: accepted and fixed. Both halves corrected: all three introduced hazards are unacceptable, and the re-assessments were made in the fix rounds that followed the review rounds which found the defects. The reviewer's parenthesis is the reason this was not deferred — the checklist is where the owner reads what is still owed, so an understatement there is worse than the same understatement inside the register.

**finding-58**: `docs/plans/2026-09-09-org-node-risk-analysis.md` was last touched in fix round 5 and carries **no fix-round-6 entry at all**. Three consequences. (i) The new test `unverified_revocation_leaves_the_record_in_place` is named nowhere in the plan; its red→green mutation attestation exists only in a commit message, not in the "Red → green attestations" section the plan designates as where each `red -> green:` line is recorded — and a signed squash will not necessarily carry that message forward. (ii) `:160` still describes `org-node/tests/service_stories.rs` as carrying only "the abnormal case of REQ-uxv2x2 added by T6", when it now carries two. (iii) `:149` says the register has "sixteen" not-minted controls; it has seventeen. [ranked: evidence]
disposition: accepted and fixed, and (i) is the one that mattered: the plan is the durable holder of the red → green attestations, this record copies them from it, and an attestation living only in a commit message is an attestation that does not survive the squash. The fix6 entry is now in the plan in the same shape as the others, and the red → green table above carries its row.

**finding-59**: `org-node/.guardrails/config.yaml:71-75` records "Measured 2026-09-09 … 50 passed, 0 failed — … service_stories 2 …". Fix round 6 added a third test to `service_stories.rs`, so the tree yields **51 passed, service_stories 3**. The unit config's own record of its gate is one test behind the tree it gates. [ranked: evidence]
disposition: accepted and fixed, with a figure the fix round measured itself rather than copied. A config comment is the unit's tool-qualification evidence and the first thing the next reader trusts, so a stale count there outlives every plan — which is the same reasoning that produced the dated measurement in the first place, one round too early to include this test.

**finding-60**: `org-node/src/transport/wire.rs:14` and `:19` document `genesis_snapshot` as "`None` for non-admission messages (e.g. revocations)". That is false — `revoke_member` sets it (`org-node/src/service.rs:1213-1214`) — and this change's own analysis depends on the opposite fact: the frame-bound entry argues an S3/P2 introduced trigger from both send paths carrying the whole pre-change record as a snapshot, not-minted control 15 rests on it, and the unit glossary's *Wire message* entry states the truth. So the change establishes that a doc-comment on a path it assesses is wrong and records that nowhere — no problem report, no dated note. PR-hvg2dy is this change's own precedent that a wrong doc-comment on a security-relevant path is a class C defect worth a report. [ranked: open work]
disposition: accepted and filed as **PR-w88sr9** (`affects: RC-gfn6kr`, opened 2026-09-10, open), with a dated fix item on the setup checklist and its age limit, and the problem-report counts corrected from four to five wherever they appear. The comment itself is deliberately not edited: this change touches no source behaviour and no source comment beyond what it already has, and the fix belongs to a `resolve-problem` change — the same line this change drew for PR-hvg2dy. The reviewer's invocation of that precedent is exactly right, and finding it means the change's fifth problem report was found by the review of the change that created the need for it.

**finding-61**: the P1 argument for the spurious self-delete enumerates one route to the store state it needs — the branch at `org-node/src/service.rs:1048-1060`, where no persona's device key is in the committed trie — and concludes "the admission of this node does not leave that behind". A second route runs through the ordinary admission path: `PersonaRecord.org_id` is a single `Option<OrgId>` (`org-node/src/store.rs:18`) and `receive_and_verify` overwrites it (`:1110`), so admitting the same persona to a second Organisation leaves the *first* Organisation's record unlinked; a later Change set for that first Organisation reaching `receive_and_self_delete_if_revoked` finds no candidate at the presence filter (`:1330`) and deletes a record the node is genuinely a member of — a stronger instance of the harm than the one enumerated, where the node was never a member of the deleted Organisation. Nothing in the code prevents one persona from being admitted twice; "one per org" is stated only in a comment. The verdict is unaffected, but the enumeration behind P1 is incomplete and the argued harm understated. [ranked: evidence]
disposition: accepted and fixed: the second route is enumerated, named as the stronger instance because the node deletes a record of an Organisation it really belongs to, the probability re-argued with both routes on the table, and not-minted control 17's wording checked against the widened hazard. This is the best finding of the round on its merits — the reviewer read past the code the entry cited and found a path the author had not, in the entry the previous round had just rewritten.

The eight findings below the bar are recorded here with their dispositions and
were **not** fixed, deliberately. Each is a wording or precision matter that
would not mislead a reader on safety, evidence or open work, and this change
has demonstrated that a round of rewriting costs more defects than it repairs.
They are the next change's inheritance, and they are written out in full so
that inheritance is legible rather than lost.

**finding-62**: three bare continuation citations in the register dangle onto the wrong antecedent file. `:864-871` and `:1254-1261` at `org-node/docs/risk/2026-09-09-org-node-hazards.md:556` follow a `org-node/src/transport/wire.rs` citation; `:1299-1310` at `:600` follows `org-node/src/transport/mod.rs:47`. All three are `service.rs` ranges and all three are correct for `service.rs`, so a reader following the file's convention lands past the end of a 42-line file. [below the bar]
disposition: accepted, not fixed. The ranges are right and the file they belong to is unambiguous from the sentence; the convention of a bare `:NNN` continuation is the register's own throughout, and rewriting three of them invites the transcription error the last four rounds kept finding. Recorded for the next change to correct with the citation pass it will already be doing.

**finding-63**: `org-node/docs/risk/2026-09-09-org-node-hazards.md:510-512` says "three have an entry below, and the fourth (RC-b6mydy) introduces none today and is noted" — and then four bullets follow, RC-b6mydy's among them. Self-contradictory on its face; the intent is plain. [below the bar]
disposition: accepted, not fixed. The four bullets are correct and the sentence's arithmetic is a leftover from an earlier arrangement of the section; a reader who counts the bullets is not misled about any hazard.

**finding-64**: the new test leaves the mock chain at the epoch B has already committed, so the forged envelope is refusable on two independent grounds — bad signature at step 2 and stale epoch at step 7. The test's discriminating power therefore rests on its error-kind assertion. I checked that it does discriminate (deleting the signature check yields `StaleEpoch`, which reddens that assertion), so the annotation holds; advancing the chain first would have tested the signature clause alone. [below the bar]
disposition: accepted, not fixed, and the reviewer's own verification is why: the assertion discriminates, so the test verifies what it claims. Advancing the chain would make it a narrower and slightly better test, and that is worth doing when the file is next touched rather than in a final fix round.

**finding-65**: `…hazards.md:1039-1042` says the abnormal-input case "**is the reason** a rejected message cannot reach the delete branch at all." The reason is the `?` on verification at `org-node/src/service.rs:1322`; the test is the evidence. The register draws exactly this distinction two pages earlier ("a test is not itself a risk control"). [below the bar]
disposition: accepted, not fixed. The distinction is real and the register is inconsistent with itself by one word; no reader is misled about what the code does or what the test shows.

**finding-66**: "the cached reader is constructed in exactly one place, `org-node/tests/chain_genesis_e2e.rs`" (`…hazards.md:224-226`, repeated in the problems ledger). One file, two construction sites. The substance of the round-6 correction is right: I searched the whole tree myself and `OnChainReader` appears in production only as its own definition and a `lib.rs` re-export, in no `service.rs` path, and not in `app/` or `preflight.rs`. [below the bar]
disposition: accepted, not fixed. "One place" means one file here and the reach claim it supports is exactly right, independently confirmed by the reviewer's own search.

**finding-67**: the dated notes appended to PR-hvg2dy are out of chronological order — a 2026-09-10 note precedes a 2026-09-09 one — in a change whose stated reason for keeping the emptied org-members pointer file is "so that the ledger's history reads chronologically". [below the bar]
disposition: accepted, not fixed, and it is a fair catch on the change's own stated value. Both notes are dated, so the history is readable; reordering prose inside an open report at the last moment is exactly the kind of edit that has gone wrong in this change before.

**finding-68**: `docs/plans/2026-09-09-org-node-risk-analysis.md:87-89` still calls REQ-ysyu9g and REQ-q92yac "the provider-side halves of RC-6a2dke and RC-wqgm2p's residuals" — the framing round-6 finding 51 rejected and the register corrected. The two register passages were fixed; the plan's record of the owner's decision was not. [below the bar]
disposition: accepted, not fixed. The register is the authority on what the expectations are halves of and it is now correct; the plan's sentence records what the owner decided on 2026-09-08 in the words used then, which is the plan's job. The distinction the register now draws is the one a reader needs, and the next change's checklist item for the expectations carries it.

**finding-69**: the ledger files were finalised to merge-dated names on 2026-09-09 while fix rounds ran to 2026-09-10, so the dated names are not the merge date the ledger README defines them as. I checked `finalize-docs.sh`: it renames only `DRAFT-*.md`, so re-running it at merge is a no-op and the roughly thirty cross-document citations of the dated names stay valid. The discrepancy is in the convention, not the references. [below the bar]
disposition: accepted, not fixed, and already recorded under Gaps below with the same reasoning the tooth-2 change used on 2026-09-08. The reviewer's check that re-running the script is a no-op is the useful part: it means the names cannot be corrected mechanically at merge, and correcting them by hand would break roughly thirty citations to fix a one-day skew this record explains.

## The final pass

After the last fix round a combined pass ran the gate over all four units and,
separately, an independent focused check of that round's five changes — a full
eighth review was deliberately not commissioned, because rewriting had been
this change's largest source of new prose defects and the scope that needed
checking was five known edits.

The gate returned PASS with the figures in the table above. The focused check
verified all five dispositions against the code, reading every line each one
cited, and confirmed the second route behind the spurious self-delete's
probability is real and code-supported. It found two residual defects, both
cosmetic and both in this change's claims about itself: a sentence saying the
change "does not touch crate source or its comments", which is false in the
letter because the test relocations and the fixtures module's own doc-comment
are under `org-node/src`, and one stale report count in a planning table. Both
were corrected directly, and the trace gate re-run clean afterwards.

It also let one pre-existing looseness stand, and recorded why: the setup
checklist speaks of "hazards its own controls introduce" where the register's
own counting test makes one of the three a further hazard and two of them
further triggers. The register's residual-risk section is the careful one and
says "entries"; the checklist's sense is the looser one the register's own
section heading uses.

Its verdict: nothing found would mislead a reader about the software's safety,
the evidence behind a claim, or what work remains open.

## Gaps

What this change did NOT establish.

- **The ledger files are dated a day before the merge.** `finalize-docs.sh`
  ran on 2026-09-09 and named the three org-node ledger files for that date;
  five review rounds and four fix rounds then carried the change past
  midnight, so the merge lands on 2026-09-10. Both ledger READMEs say the
  dated name is the merge date. The files keep the names they have, and every
  cross-reference in this change and in org-members' register points at those
  names, so renaming them now would mean re-editing every pointer to fix a
  one-day skew whose explanation travels with this record. The same judgement
  was made and recorded for the tooth-2 change on 2026-09-08.

- **Decision coverage is still unmeasured, and org-node has no coverage at
  all.** Class C requires statement AND decision coverage. org-members'
  statement and region floors held (figures above), its branch column reads
  `0 0 -`, and org-node and app have no `coverage_command` — a gap against a
  mandatory requirement, not a decision, carried on the setup checklist since
  2026-09-01 and untouched here. This change moved org-node's tests into the
  gate's sight; it did not measure them.
- **None of the five defects this change filed is fixed.** PR-vt244s (publish
  before persist), PR-2dmjzj (loopback admission unbound to the joiner's
  Device key) and PR-u4c2vp (the revocation receive path commits from an
  unchecked sender) are open with their reproducing tests specified and not
  written; PR-w88sr9 (a wire-message field's doc-comment contradicts the path
  this change assesses) is open with the correction specified; and PR-hvg2dy,
  moved into this ledger, is open on its original grounds. Their hazards
  therefore carry the residual risk the register states, and their fixes are
  in the not-minted list. The age limits fall on 2026-10-02 for PR-hvg2dy,
  2026-10-09 for the two the survey found, and 2026-10-10 for the two the
  reviews found. Five open reports against a `problem_open_max` of 10, so the
  budget holds, and every one has a dated item on the setup checklist.
  Filing rather than fixing was the owner's decision on 2026-09-08, on the
  ground that a hazard analysis and a source fix do not belong in one change.
- **Eight findings from the last review round are recorded and not fixed.**
  They are findings 62 to 69 under Review above, each ranked below the merge
  bar by the reviewer that raised it and each carrying its disposition and
  its reason. Three concern citation convention, three are single words or
  arithmetic leftovers in the register's own prose, one is note ordering
  inside an open report, and one is the ledger date skew below. None would
  mislead a reader about safety, evidence or open work. They were left because
  this change has demonstrated the opposite risk: five of the seven review
  rounds found defects that a previous fix round had introduced, so a
  narrower final round was judged safer than a tidier one.
- **Nine of the register's hazards have no minted identifier**, and
  the four hazards the controls introduce are assessed in prose. The reason is
  the ledger limitation org-members' register documents at length: the gate
  refuses a hazard whose control is not implemented and tested, so a hazard
  whose control is a deployment obligation, another unit's work, or an
  unwritten fix cannot be given an ID. Every one is written out with its
  severity, probability and control, so nothing is lost to a reader; what is
  lost is the mechanical trace, and that is a toolkit limitation to report
  upstream.
- **Two controls' decisive half is an open expectation on a provider.**
  RC-6a2dke rests on on-chain-client returning the Finalised Organisation
  state (REQ-ysyu9g) and RC-wqgm2p's residual on org-members rejecting a
  device removal that leaves the Member-as-a-group key (REQ-q92yac). Both
  were opened 2026-09-06, both are warnings until 2026-12-05, and neither
  requirement carries `(implements: RC-…)` in this change — deliberately: an
  unmet expectation that implements a control fails the unit's gate on every
  run, and the providers' 90 days have not run. The annotation is the
  follow-up's, and until it exists the escalation the toolkit intends is not
  armed.
- **The fuzz evidence is a one-second smoke run.** Both bolero targets run
  under the default generative engine at the merge gate, print no pass/fail
  counts, and exit on `max duration exceeded` after about 190,000 iterations
  each. There is no seed corpus and no libFuzzer lane. REQ-9g6as6 and
  REQ-bcxz96 are annotated to them and were not reddened by mutation — they
  predate this change, which adds only the annotation.
- **The chopsticks lane still runs nowhere, and there is no reorganisation
  test.** The three chopsticks-dependent targets are excluded from
  `verify_commands` and from CI, so the finalised-publish behaviour the
  register discusses is exercised by nobody at any gate; and no test in the
  crate reorganises a chain (the helper named `chopsticks_reorg.rs` only mines
  blocks). That is why the discarded-root hazard is prose with no minted
  control.
- **No architecture, no SOUP, no LLRs for this unit.** `doc_sad` holds a
  README and an empty SOUP table. Class C requires LLRs per software item, and
  the fifteen REQs written here have none beneath them, so every test
  annotates a REQ directly. Tooth 4.
- **CI has not run.** The four new test targets were added to `rust.yml`'s
  org-node step and to `verify_commands`, and both were exercised at this
  change's gate on this machine only. Nothing was pushed, so the reshaped step
  is unconfirmed in CI — the same open item the 2026-09-05 checklist carries.
- **The severities and probabilities are the owner's confirmed judgement, not
  a measurement.** Every hazard is S3 on the harms the class C ADR names, and
  the P1/P2 split turns on whether an ordinary event or a deliberate act plus
  a precondition is needed. The situation-to-harm step is not estimated
  anywhere, as org-members' register states for its own numbers.
- **The overall residual-risk verdict is a reasoned conclusion, not the output
  of a stated criterion.** ISO 14971 clause 8 wants overall acceptability
  criteria in a risk management plan; this project has none. Same open item as
  org-members' register named on 2026-09-02.
- **CI's clippy job does not cover org-node, and no configured gate runs
  clippy on it at all.** That is why the review, not the gate, found the
  panic-lint regression this change introduced: the job lints org-members and
  on-chain-client only, and org-node's `verify_commands` compile the crate
  without `-D warnings`. The regression is fixed and both invocations now
  exit 0, but nothing mechanical would catch the next one. Adding org-node
  and app to the clippy job is a checklist item this change opens.
- **`app` ran as a dependent and has no items of its own.** Its gates ran
  because it depends on both touched units; its ledgers are still empty and
  its own risk analysis is a later change, as are on-chain-client's. Tooth 3
  is finished only for org-node.
