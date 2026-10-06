# Verification — org-members, on-chain-client, org-node and app switched to `person`'s identity types (2026-10-06)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line.

branch: worktree-worktree-person-shared-types
reviewer: independent agent reviewers, one fresh subagent per round, each with no implementation narrative — round 1, round 2, round 3 (whose two helpers wrote the partial reports 3a and 3b before master moved, then a full round on the merged tree) and a narrow round 4 over the trim; every round except 3b ran the suite itself in a nested review worktree and reported its counts
verdict: rounds 1 and 2 reject; round 3 approve with findings; round 4 (narrow, ef6e261..1fe6a12) approve with findings. Every finding below carries a disposition; the round-4 fixes (95fcf59) went to no further reviewer, under the stop rule stated to the owner before round 3: a round that finds only record or test gaps, closed by a fix and a mutation proof, ends the review; anything needing a ruling goes to the owner first (round 4's two such findings did)
reproduced: yes for the one defect this change fixes in its own terms — PR-b7khyw (org-members accepted small-order, non-canonical and mixed-order ed25519 keys): `org-members/tests/device_key_decoding.rs` first asserts that `VerifyingKey::from_bytes` accepts each input, and its five tests failed against the pre-merge `person` (aff5e6b), the constructor accepting all four inputs; `org-members/tests/member_key_decoding.rs` (characterisation tests) failed 8 of 8 against master 05f6f04, where the Member-as-a-group key was still an ed25519 `VerifyingKey`. finding-92 (round 4: a pending first admission pre-empted by a relayed admission) was reproduced by the reviewer and is deferred, not fixed. PR-vkw22m and PR-u4c2vp were not reproduced: the first is moot (no signature is checked), the second is resolved by ruling

Change: org-members, on-chain-client, org-node and the app use `person`'s
identity types; Member-as-a-group keys and the on-chain Organisation public key
are X25519; the Envelope carries no signature; a created Organisation publishes
the X25519 key of a fresh Organisation private key held in the creator's
encrypted store. Branched from `master` at `f688a3c` (the `person` merge).
Plans: `docs/plans/2026-10-04-person-shared-types.md` (scope, owner rulings),
`docs/plans/2026-10-04-person-shared-types-implementation.md` (T8, T9's
org-members steps), `docs/plans/2026-10-05-org-node-organisation-public-key.md`
(T1–T9) and `docs/plans/2026-10-05-switch-trim.md` (T1–T13, owner answers
Q1–Q4).

Units touched: org-members, on-chain-client, org-node, app; `person` untouched
(its code merged in `f688a3c`). Also `on-chain/POST_POC.md` (not a unit; a
dated note), the root `docs/CONTEXT.md` and `Cargo.lock`.
Impact set (`check-units.sh --impact master..HEAD`): org-members,
on-chain-client, org-node, app.

Master merged during the change: `05f6f04` (e8b7d8d), `1feb608` (a18e5d5) and
`06c357b` (8b4b4de), each followed by the merge items it surfaced (below,
m-1 to m-16 for `1feb608` and `06c357b`; `8ddd99a` and `9c11f89` for
`05f6f04`).

## The gate

**Final gate, after merging master `d8b9f9b` (the app architecture change),
2026-10-06.** Run by a fresh subagent at tree
`380cbf695c625856f356e6f7fa1ef2b16ec149bc` (HEAD `ef86385`), unchanged from
start to end, `git status` clean. All 39 steps exited 0, read from counts:
person 73 passed, clippy clean; org-members 241 passed, 1 ignored
(`preflight_probe`, by design), quint 3 typechecks, 63 tests, mbtInv no
violation; on-chain-client 73 passed, 3 fuzz harnesses clean; org-node 186
passed, 3 fuzz harnesses clean, quint 2 typechecks and 5 invariants no
violation, chopsticks targets built; app 135 cargo tests (connection_status
10, csp_policy 24, ipc 33, org_id_parsing 14, receiver_events 32,
receiver_guard 6, startup_policy 13, state_assembly 3), `npm run check` 0
errors and the known `node` types warning, vitest 36 passed. check-trace and
check-ids exit 0 for all five units; check-units clean (562 tracked paths);
check-review clean (100 findings). Coverage identical to the gate below:
person 100/100, org-members 96.29 % lines / 95.33 % regions, on-chain-client
41.75 / 42.60 (its accepted floor). The Implements map was byte-identical to
the gate below. Totals: 708 Rust tests passed, 0 failed, 1 ignored.

After this gate, two documentation-only edits were made, and the trace, ID and
unit gates were rerun on them (exit 0): LLR-7bk6qh amended in place (finding
101) and the app config's count comment. No code or test changed.

The earlier final gate, on the tree before that merge, follows.

Every figure is measured on the tree named in the summary below. No figure is
copied forward from an earlier round; each review round's reviewer measured
its own tree (counts quoted with the round headings).

Ordering, recorded: `finalize-docs.sh` ran (22db88c, "chore: finalize ledger
files") before the first full gate of the merge sequence, not after it as
`merge-change` orders the steps. The gate below measures the finalized tree.

Final gate, run by a fresh subagent on 2026-10-06 at tree
`a901341d8a144a6d383f80d5d4b840f2cdf5c9e1` (HEAD `777d68b`; the tree was the
same at the start and end of the run, and `git status` was clean). Every
command exited 0, and every result below is read from its counts.

| Unit | Command | Result |
|---|---|---|
| (all) | `check-units.sh` | clean: 5 units, 6 disclaimed, 552 tracked paths |
| person | `cargo test -p person`; clippy `-D warnings` | 73 passed, 0 failed; clippy clean |
| org-members | `cargo test -p org-members` | 241 passed, 0 failed, 1 ignored (`preflight_probe`, ignored by design) |
| org-members | quint 0.33.0: 3 typechecks, `quint test`, `quint run mbtInv` | clean; 63 passing; no violation |
| on-chain-client | `cargo test` (own manifest) | 73 passed, 0 failed; 3 fuzz harnesses ran 1 s each, no crash |
| org-node | `cargo test` (all configured targets) | 186 passed, 0 failed; 3 fuzz harnesses ran 1 s each, no crash |
| org-node | quint: 2 typechecks; forkSafety, revocationSafety, revokedExcludedFromOrgSecret, tauWindow, convergence | clean; no violation in any |
| org-node | chopsticks targets `--no-run` | built |
| app | `cargo test` (src-tauri) | 84 passed, 0 failed |
| app | `npm run check`; `npm run test` | 0 errors, 1 warning (tsconfig: no type definition for `node`, present before this change); 30 passed |
| each unit | `check-trace.sh`, `check-ids.sh --allow-draft-files` | exit 0, no errors; warnings only (open problem reports and unmet expectations, all within limits) |

Totals: 687 Rust tests passed, 0 failed, 1 ignored; 6 fuzz harnesses clean;
63 quint tests and 6 invariant runs clean; 30 vitest tests passed.

Coverage, against the floors: person 100.00 % lines and regions (floor
99/99); org-members 96.29 % lines, 95.33 % regions (92/91; its run leaves out
the conformance suite); on-chain-client 41.75 % lines, 42.60 % regions
(41/42, its documented accepted shortfall). org-node and the app have no
coverage command. Decision coverage is not measured by machine in any unit.

The shared cargo cache `/tmp/cargo_home_fuzz` was found corrupted on
2026-10-06 (about 550 unpacked crate sources missing; the archives intact).
The final gate and the round-4 fix task ran with a working copy in the
session scratchpad; the shared cache was not touched.

One-sided robustness, judged by the dispatcher:
- RC-95dgg8 has no direct `verifies:` test. It is realised by REQ-txvtm9,
  which declares `implements: RC-95dgg8`, and REQ-txvtm9's tests cover both
  the normal and the abnormal case. Accepted, as for every control realised
  through a requirement.
- REQ-ztdza4's tests all expect a commit. Its abnormal input is an unusual
  sender, which by the owner's ruling commits; there is nothing for a
  refusal-side test to show. Accepted.
- LLR-xq9nrq has one test. The abnormal side of the no-Invite path is LLR-mbjfq8's
  `a_first_admission_with_no_imported_invite_that_misses_the_chain_root_commits_nothing`. Accepted.
- REQ-54txzh carries REQ-2qa5r5's documented exemption: it constrains a
  declaration and has no runtime input domain. LLR-hezpr7's abnormal side
  is its sibling LLR-sq76u3, as its architecture file states. Accepted.
- REQ-8jb4ny and LLR-z954wj have their normal case asserted inside tests
  named for refusals. Recorded, not changed.

Problem-ledger delta: opens PR-ve9zw8 (org-node). Resolves PR-b7khyw
(org-members), PR-u4c2vp and PR-vkw22m (org-node). PR-szkat6 (org-node) was
resolved at the `1feb608` merge and reopened by the trim, restated as its open
part: net, still open.

## Red → green

Copied from the plans' Done and status blocks and from the commit messages on
the branch. "Compile" means the test target failed to build because the item
did not exist. Where the only evidence is a mutation proof or a
characterisation test, the row says so; where none was recorded, it says that.

| Item | Test | Watched red |
| --- | --- | --- |
| LLR-z954wj (org-members) | `device_key_decoding.rs` (public constructor, one DevicePublicKey, a set, a member record, a change set) | five tests failed against the pre-merge `person` (aff5e6b), constructor accepting all four inputs; each test first asserts `VerifyingKey::from_bytes` accepts its input. Round 3b added off-curve, order-8, y = p and y = p − 1 cases with exact errors (6e34739). The non-canonical clause is an equivalent mutant (Gaps) |
| LLR-a645bx (org-members) | `member_key_decoding.rs` | characterisation tests: 8 of 8 failed against master 05f6f04 (ed25519 Member-as-a-group key) (9df956b); mixed-order acceptance test added in round 2 (ae363f4) |
| LLR-st6j2r (org-members) | `device_slots_parse_holds_keys_sorted`, `device_slots_parse_refuses_too_many_and_repeated_keys` | none recorded: replacement wording of LLR-t3p9zk for `person::DeviceSlots`, tests carry both IDs (6e34739) |
| LLR-jfj6pc (org-members) | `the_device_empty_sentinel_encodes_no_device_public_key`, `the_sentinel_check_tells_a_device_public_key_from_other_bytes` | none recorded (6e34739) |
| LLR-k6dhz7, LLR-t3p9zk, LLR-377ddr, LLR-pys2ek, LLR-w5nkbu, LLR-xyv6p9, LLR-68tka5, LLR-72p8bz, LLR-kdhd2v, LLR-sa3ugj (org-members, amended) | the existing suites; `encoding_golden.rs` | T8 is a refactor: no red, suites identical to baseline, `GOLDEN_ALICE_LEAF` and `GOLDEN_ROOT` unchanged (5cb04f8, 4416c0d). T9 fixtures (9e93bfc): before, 180 failures (encoding_golden 0/3, fuzz_tests 1/9, integration_test 24/161, mbt_conformance 2/32, newtypes 14/16); after, identical to baseline. Golden values re-derived because the old Member-as-a-group key fixture is not a canonical X25519 key (`GOLDEN_ROOT` `2e81afbd…a19d` → `4740435a…efd4`; LLR-377ddr amended to cite it) |
| REQ-54txzh, LLR-hezpr7 (on-chain-client) | `type_widths.rs`, `decode_org_state.rs` | none: wording of existing byte-level decoding (supersede REQ-2qa5r5, LLR-nq7nhg); annotations added, no behaviour change (75dc6ca) |
| REQ-ech45n, LLR-sj7cd5, LLR-3fwykc, LLR-2dvhz8 | the `organisation_key.rs` tests | org-node plan T6: compile (four E0609 `no field org_private_key`); then each of five failed on its assertion with `create_organisation` still publishing the interim key; `the_organisation_secret_is_not_in_the_record_debug_output` failed with the derived `Debug` ("the Organisation secret is printed"). LLR-sj7cd5's "before the chain is written": mutation M17b killed (d607c17). LLR-2dvhz8's unset case added in round 4 (95fcf59) |
| REQ-8jb4ny, LLR-3jjgtw | `an_organisation_public_key_that_is_not_a_valid_x25519_key_is_refused`, `an_organisation_state_read_with_an_invalid_key_is_refused`, `a_receive_against_a_state_with_an_invalid_key_commits_nothing` | org-node plan T7: compile (E0432 `OrgPublicKey`, E0599 `from_chain`, `InvalidOrgPublicKey`); mutation `OrgPublicKey::parse` accepting every input failed all three |
| LLR-98ufry | `x25519_public_key_matches_rfc_7748`, `x25519_seed_round_trip_preserves_key`, `x25519_debug_does_not_print_the_secret`, `an_x25519_key_pair_cannot_be_cloned`, `an_x25519_key_pair_declares_zeroize_on_drop_and_zeroize_clears_its_secret` | org-node plan T3: compile (six E0433 `X25519Keypair`). Clone clause: a compile-time trait probe (95fcf59). Drop clause: inspection only — mutation M19 survives (Gaps) |
| LLR-mmdu38, LLR-56hc77, LLR-bwb9pu, LLR-e58j8m, LLR-ctzkv7, LLR-s7yu4k (amended) | `keys` tests, `key_custody.rs` | T3 compile red as above; Q4 folds are annotations only (trim T6: no red) |
| REQ-txvtm9, RC-95dgg8, LLR-9f5hmr | `a_sequence_number_other_than_the_chain_epoch_is_refused`, `a_sequence_number_beyond_the_chain_epoch_cannot_jam_the_receive_path`, `…_the_self_delete_path`, `the_sequence_number_is_checked_after_the_stale_epoch_and_before_the_root_match`, `the_admission_envelope_carries_the_epoch_its_update_produced` | the round-1 fix (c19973b) records no red line; LLR-9f5hmr's order: mutation M21b killed (d607c17) |
| REQ-ag6kqm, REQ-nhe2zu, LLR-na7p4w, LLR-9fvb3y, LLR-mcdh85, SDD-kk2y3e, LLR-e7s4ye, LLR-p8uu47, LLR-ybn5pr, LLR-cs4mpb, LLR-9sknpa, LLR-pzde8b (amended) | `verify_against_chain.rs`, `envelope_binding.rs` | org-node plan T4: compile (E0432 `Envelope`, E0560 `sender`); trim T1: `undecodable_change_set_bytes_are_refused_as_malformed_delta` and every `ctx(org)` call failed to compile before `VerifyContext` lost its sender fields; `a_malformed_change_set_from_a_device_outside_the_record_is_refused_as_malformed` (`UnknownSender` where `MalformedDelta` asserted) |
| REQ-ztdza4, LLR-3q63zv, LLR-6dc598, LLR-tax3pm, LLR-8hdu9x, SDD-72ddm6 (amended) | `a_revocation_relayed_by_a_non_member_is_still_acted_on`, `pr_u4c2vp_an_update_relayed_by_a_non_member_is_committed_on_the_self_delete_path`, `revocation_from_an_unknown_device_leaves_the_record_in_place`, `update_relayed_by_a_non_member_after_admission_is_committed`, `a_removal_relayed_by_the_member_it_removes_is_committed` | trim T1, T3: each failed with `UnknownSender` (or `UnknownSender` where `StaleEpoch` asserted) before the sender check was removed |
| REQ-xa6smf, LLR-mbjfq8, LLR-xq9nrq, LLR-rys5nx, LLR-j83kc8, LLR-e5c9ud, SDD-8cpyfa (amended / new) | `first_admission_from_a_device_other_than_the_invites_admin_is_committed`, `a_first_admission_with_no_imported_invite_rests_on_the_chain_alone`, `a_first_admission_with_no_imported_invite_records_the_chains_key_as_admin_member_key`, `a_first_admission_whose_invite_names_another_organisation_key_is_committed`, `a_first_admission_with_no_imported_invite_that_misses_the_chain_root_commits_nothing` | trim T2: failed with `UnknownSender`, `NoImportedInvite`, `NoImportedInvite` and `InviteOrgKeyMismatch` respectively before the first-admission arm lost its checks. LLR-rys5nx's chain-key clause: mutation M12 (record the Invite's key) now fails the fourth test (95fcf59). The fifth (round 4) characterises an existing refusal; no red recorded |
| LLR-8bum44 (amended) | `an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored` | trim T4: failed (`InvalidInvite` where `Chain("blob decode…")` asserted) before `import_invite` dropped its `map_err` |
| LLR-322xfu | `the_organisation_private_key_is_a_secret_type_and_the_only_way_to_its_key_pair`, `the_organisation_private_key_and_its_key_pair_never_render_its_bytes` | none recorded (b77e125) |
| LLR-ayrdr8 (amended) | `encoding_golden.rs` (org-node) | not a red: the new pinned values are derived from master's bytes (kept in a comment) for the two deliberate format changes, not captured from the code (b4fccf8) |
| REQ-kn5rtx, RC-3rddh7 (app, amended) | `receiver_events.rs` (`an_invalid_organisation_public_key_is_classified_as_a_receiver_error` and the verdict tests) | org-node plan T8: compile (E0053 at `state.rs`, E0599 `BadSignature`); round 3a: tests that four refusals do not stop the receiver loop killed M5 (ac3c206); trim T5: APP-GATE failed to compile (`no variant named UnknownSender`, `InvalidInvite`, `InviteOrgKeyMismatch`, `NoImportedInvite`) before `classify_receive_error` dropped them |
| Other amended org-node items (SDD-sxp8hb, SDD-na9nc3, SDD-rx2yvy, SDD-z85ux9, LLR-g9vmbx, LLR-rv4vux, LLR-rb8r65, LLR-ghja3x, LLR-vdyu65, LLR-zj88e6, LLR-9zfnmb, LLR-u6rq4s, LLR-37cj3n, LLR-y2v8v2, REQ-hzm4kt, RC-pm9kmx, RC-b6mydy) | through the tests above | wording of behaviour the rows above made red; no separate red |

## What was wrong, and what was built

On master after `f688a3c`, `person` existed but nothing used it: org-members
held its own copies of the identity types, accepted small-order, non-canonical
and mixed-order ed25519 keys (PR-b7khyw), and validated Member-as-a-group keys
as ed25519. org-node signed every Envelope with one ed25519 key and verified it
under the chain's `orgPubKey`, which doubled as an administrator's key
(PR-szkat6), with a non-strict verify (PR-vkw22m).

Built:
- org-members uses `person`'s `Name`, `Surname`, `DevicePublicKey`,
  `PersonPublicKey`, `DeviceSlots` and device sub-trie; every DevicePublicKey from
  outside is parsed by `person`'s rule (LLR-z954wj), every Member-as-a-group
  key by its X25519 rule (LLR-a645bx); the device-trie empty sentinel encodes no
  DevicePublicKey (LLR-jfj6pc). Member hashes are unchanged over the same bytes;
  the golden fixtures moved to canonical X25519 keys.
- on-chain-client states `orgPubKey` as the Organisation public key, an X25519
  key-agreement key, read as bytes; org-node parses it (REQ-54txzh,
  LLR-hezpr7).
- org-node: X25519 Member-as-a-group keys from `member_seed`; the Envelope
  loses its signature and nothing about the sender is checked; the Sequence
  number must equal the chain epoch (REQ-txvtm9, RC-95dgg8); the Organisation
  state's key is parsed by `person`'s X25519 check (REQ-8jb4ny); a created
  Organisation publishes a fresh key, distinct from every genesis key, its
  private half kept in the encrypted store, never printed (REQ-ech45n,
  LLR-sj7cd5, LLR-3fwykc, LLR-2dvhz8, LLR-322xfu); X25519 secrets are not
  Clone and zeroize on drop (LLR-98ufry). Master's signature items are amended
  in place (trim convention).
- app: the verdict list matches `classify_receive_error`; `BadSignature` is
  gone, `SeqNotEpoch` is a verdict, `InvalidOrgPublicKey` a receiver error.
- Glossary: root **Organisation public key** entry (`docs/CONTEXT.md`); org-node
  glossary entries for Device key (the earlier term for a DevicePublicKey) and
  Organisation private key.

The change was first written with a sender-device check replacing the
signature (REQ-7h7qp3, RC-2e6k44) and a mandatory, chain-checked Invite. The
owner's chain-authority rulings of 2026-10-05 removed both; the trim
(`docs/plans/2026-10-05-switch-trim.md`) withdrew those items before merge and
turned this branch's supersessions into in-place amendments.

Owner rulings taken during this change:
- 2026-10-04: `orgPubKey` becomes the Organisation's X25519 key; Member,
  Person and Organisation keys used for key agreement are X25519; order of the
  work (person, org-members, on-chain(-client), org-node, in this change).
- 2026-10-05: validated types construct through `parse` (call sites renamed at
  the master merge).
- 2026-10-05: the Envelope signature is dropped. Admin authority over on-chain
  updates is the on-chain multisig proxy, outside org-node.
- 2026-10-05 (round 1, finding-1): the Sequence number equals the epoch of the
  Organisation state the receiving node read.
- 2026-10-05 (round 1, finding-4): the unauthenticated Organisation secret is
  assessed in the risk file and recorded as PR-ve9zw8.
- 2026-10-05 (at the `1feb608` merge): master items this branch contradicts
  are superseded and their reports resolved (supersede-and-resolve). Replaced
  the same day by the chain-authority ledger convention: retired behaviour is
  amended in place with a dated note.
- 2026-10-05 (chain-authority rulings, after round 3): nothing about the
  sender of an Envelope is checked; the chain root at a newer epoch is the sole
  authority. The owner chose option 1, trim this branch to what both designs
  agree on, and answered Q1 (`admin_member_key` from the imported Invite, else
  the chain's Organisation public key), Q2 (merge alone; record the interim
  gap; PR-ve9zw8 stays open, retargeted to any peer), Q3 (PR-u4c2vp resolved by
  ruling) and Q4 (fold the four KEEP replacements into their old items).
- 2026-10-05 (round 4): finding-1 deferred to the chain-authority session;
  the stale parent links of LLR-ctzkv7, LLR-g9vmbx and LLR-9zfnmb left to
  chain-authority's change 1.
- This change is finished and merged before chain-authority, which builds on
  it.

Accepted risks (owner, 2026-10-05; `org-members/docs/risk/2026-10-03-key-uniqueness.md`):
one keypair reused as a DevicePublicKey and as a Member-as-a-group key has
different bytes in each role, so the byte-comparing key rule does not see the
reuse; a PersonPublicKey accepts mixed-order u-coordinates, so one X25519 key
has up to eight accepted encodings the byte comparison sees as different keys.

## Review

Four rounds. Round 3's reviewer first worked through two helpers on tree
`cb5258f`; master moved (`1feb608`, `06c357b`), the reviewer was stopped, and
its helpers' partial reports are recorded as rounds 3a and 3b. The merge items
m-1 to m-16 are not review findings: the author found them merging master and
they are numbered here so each carries a disposition. The source findings
carry no severities; none is invented here.

### Round 1 — reject; reviewer saw org-members 226 passed, 1 ignored; on-chain-client 73; org-node 131 and three fuzz targets; app 79 and vitest 30; every gate exit 0

**finding-1**: code — (round 1) Sequence number no longer integrity-bound; any accepted device commits `parent_seq = u64::MAX` and jams `StaleSeq` forever (verify.rs:67/:91, service.rs:1019/:1255); risk file 2026-10-05-envelope-authenticity.md:38-51 silent.
disposition: fixed — owner ruling 2026-10-05: the Sequence number must equal the epoch of the Organisation state the node read; REQ-txvtm9 and the risk file amended, RC-95dgg8 added (3b176eb, c19973b, f5f0d09); tests `a_sequence_number_beyond_the_chain_epoch_cannot_jam_the_receive_path` and `…_the_self_delete_path`.

**finding-2**: code — (round 1) REQ-7h7qp3's pre-decode check for a known Organisation untested at service level (mutation M6 survives, service.rs:953).
disposition: fixed in round 1 by a service-level test (c19973b); later moot — REQ-7h7qp3 was withdrawn by the trim (chain-authority ruling: no sender check).

**finding-3**: code — (round 1) LLR-p9xjze's administrator exclusion unverified (M9).
disposition: fixed in round 1 by a test (c19973b); LLR-p9xjze was later withdrawn by the trim. The exclusion is stated in LLR-e5c9ud and evidenced (round 3, finding-90).

**finding-4**: requirement — (round 1) `org_secret` adopted from any record device, not reassessed (service.rs:1019; architecture-derived.md:587).
disposition: deferred — owner ruling 2026-10-05: assessed in the risk file and opened as PR-ve9zw8 (c4aadc9, f5f0d09); after the trim retargeted to "any peer", closed by chain-authority's change 2 (receipt check against the chain's Organisation public key).

**finding-5**: record — (round 1) live items name superseded IDs (REQ-gju89b; LLR-8n95rf, LLR-cja9zv, LLR-y2v8v2 satisfy REQ-nhe2zu; six SDDs trace REQ-nhe2zu; on-chain-client SDD-5wamsz, LLR-xv7auy name REQ-2qa5r5).
disposition: fixed for org-node (f5f0d09); on-chain-client lines already name REQ-54txzh beside REQ-2qa5r5 — no change; REQ-gju89b keeps RC-pm9kmx as history — no change. The trim later amended REQ-nhe2zu in place, so those traces stand.

**finding-6**: record — (round 1) open problem reports' `affects:` name superseded items only.
disposition: fixed (f5f0d09).

**finding-7**: record — (round 1) `a_committed_admission_reaches_the_disk_and_consumes_the_invite` lacks REQ-txvtm9.
disposition: fixed — annotation added (c19973b).

**finding-8**: code — (round 1) `Invite.admin_member_key` never parsed (ADR); `Invite.org_pub_key` never compared with the chain.
disposition: fixed — parsed at import, compared on first admission (c19973b). The comparison (`InviteOrgKeyMismatch`) was later removed by the trim (owner answer Q1); the parse stays.

**finding-9**: requirement — (round 1) no LLR refines REQ-ech45n; DuplicateKey refusal, `org_private_key` field and redacting Debug unstated; DevicePublicKey collision clause unverified (M7).
disposition: fixed — LLR-sj7cd5, LLR-3fwykc, LLR-2dvhz8 and a test (c19973b, f5f0d09).

**finding-10**: requirement — (round 1) org-members' X25519 Member-as-a-group key validation has no org-members item or test at its entry points.
disposition: fixed — LLR-a645bx and `member_key_decoding.rs`, red 8 of 8 against master 05f6f04 (9df956b).

**finding-11**: record — (round 1) "Device key" used though the root glossary avoids it; org-node CONTEXT.md:14 stale; "Organisation secret" misused for `org_private_key`.
disposition: fixed (f5f0d09, 9df956b); completed in round 2 (finding-15) and round 3 (finding-89).

**finding-12**: record — (round 1) plan 2026-10-04-person-shared-types.md:92, :100-101 still says change sets are signed by an admin device key.
disposition: fixed — dated note (9df956b).

**finding-13**: code — (round 1, gate note) REQ-7h7qp3 has no normal-case test tagged.
disposition: fixed in round 1 (c19973b); later moot — REQ-7h7qp3 withdrawn by the trim.

### Round 2 — reject; reviewer saw org-members 234 passed, 1 ignored (first run 11 `mbt_conformance` failures "Quint returned non-zero code", not reproduced with `--no-fail-fast` or `--test-threads=1`); on-chain-client 73; org-node 139; app 80 and vitest 30; gates exit 0; 22 org-node mutations, 19 killed

**finding-14**: requirement — (round 2) LLR-5svrw8 and LLR-z954wj claim parents that do not state them; unassessed.
disposition: fixed — both marked derived and assessed (38b0308, ae363f4). LLR-5svrw8 was later withdrawn by the trim.

**finding-15**: record — (round 2) round-1 finding-11 incomplete: "Device key" in new org-node code and docs; master items REQ-xa6smf, REQ-ztdza4, RC-b6mydy and the verify-and-commit.md:15 pointer; soup.md:89 "Organisation secret"; "Organisation key" and "Organisation private key" undefined.
disposition: fixed — new text says DevicePublicKey; org-node glossary gains "Device key" (the earlier items' term for a DevicePublicKey; master items are not superseded for a pure term change) and "Organisation private key" (38b0308).

**finding-16**: code — (round 2) `Invite.admin_device_key` unparsed at import.
disposition: fixed — parsed as a DevicePublicKey (d607c17). The trim (T4) restored master's import error in place of `InvalidInvite`; the parse stays (owner answer Q1).

**finding-17**: code — (round 2) LLR-5svrw8's order (mismatch before snapshot decode) untested (M5b).
disposition: fixed — test, M5b killed (d607c17); later moot, LLR-5svrw8 withdrawn by the trim.

**finding-18**: code — (round 2) LLR-sj7cd5's "before the chain is written" untested (M17b).
disposition: fixed — the test asserts the chain mock untouched; M17b killed (d607c17).

**finding-19**: code — (round 2) LLR-9f5hmr's order untested (M21).
disposition: fixed — `the_sequence_number_is_checked_after_the_stale_epoch_and_before_the_root_match`; M21b killed (d607c17).

**finding-20**: requirement — (round 2) app REQ-kn5rtx rationale against nine verdicts.
disposition: fixed — app rationale amended (ae363f4); rewritten again by the trim (T5).

**finding-21**: requirement — (round 2) LLR-z954wj and LLR-a645bx claim `InvalidDeviceKey`/`InvalidPersonKey` on decode paths; tests check `is_err` only.
disposition: fixed — decode refusals asserted by message (ae363f4); exact errors in round 3b (finding-57).

**finding-22**: requirement — (round 2) LLR-a645bx's mixed-order acceptance untested.
disposition: fixed — test (ae363f4).

**finding-23**: record — (round 2) org-node soup.md :90, :103 name REQ-nhe2zu only.
disposition: fixed (38b0308).

**finding-24**: record — (round 2) org-members still describes a signature-checking consumer.
disposition: fixed — dated note and README (ae363f4).

**finding-25**: record — (round 2) RC-95dgg8 absent from HAZ-vxabf9 and the residual table.
disposition: fixed (38b0308).

**finding-26**: record — (round 2) org-node glossary: Sequence number.
disposition: fixed (38b0308).

**finding-27**: record — (round 2) stale code citations.
disposition: fixed (38b0308).

**finding-28**: record — (round 2) LLR-a645bx's assessment placed in master's dated file.
disposition: fixed — moved to a new 2026-10-05 org-members risk file; the owner-accepted narrowing stays as an amendment of the existing assessment (ae363f4).

**finding-29**: record — (round 2) SDD-d6x85b names the deleted `device_trie.rs`.
disposition: fixed — dated notes (ae363f4).

**finding-30**: record — (round 2) a risk argument rests on `remove_device` being crate-private.
disposition: fixed — restated on `with_p2p_device_slots`, `pub(crate)` (ae363f4).

**finding-31**: record — (round 2) on-chain-client pairing table and LLR-hezpr7 robustness; traces naming both IDs.
disposition: fixed for the pairing and the robustness statement (ae363f4); traces naming both IDs — no change (old ID kept beside its replacement, as in org-node).

**finding-32**: record — (round 2) golden constants re-derived without evidence.
disposition: fixed — the file comment cites plan T9's evidence ("Steps 5, 6, 8 for org-members done (9e93bfc)") (ae363f4).

**finding-33**: record — (round 2) PR-b7khyw uses `resolved:` instead of `resolution:`.
disposition: fixed (ae363f4); found not to hold in round 3 (finding-90) and fixed again (8aa8b74).

**finding-34**: record — (round 2) a test name says signing key.
disposition: fixed — renamed (3079063).

**finding-35**: code — (round 2) X25519 secrets not zeroized.
disposition: fixed — `X25519Keypair` zeroizes on drop and is not Clone (LLR-98ufry), HAZ-45ucqx residual note; the `OrgRecord` field stays under PR-hqwpg9 (d607c17, 38b0308). The Drop call is verified by inspection only (finding-76).

### Round 3a — partial (on-chain-client, app, on-chain), tree cb5258f, before the `1feb608` merge; a helper of the round-3 reviewer

**finding-36**: requirement — (round 3a) the app classifies the no-Invite `UnknownSender` (service.rs:958-965) and `InviteOrgKeyMismatch` (:975-976) as verification verdicts though both are local conditions; the RC-3rddh7/HAZ-9fmhm4 asymmetry says a misfiled verdict is the hazard; `StaleEpoch` also fails the new wording; the app's assessment of REQ-kn5rtx (app-hazards.md:1144-1155) not reassessed.
disposition: fixed — the no-Invite refusal got its own variant (`NoImportedInvite`), both classified as receiver errors, rationale covers `StaleEpoch`, app risk reassessed (ac3c206, 6b4cbb5). The trim later removed both variants (T5): org-node no longer raises them.

**finding-37**: record — (round 3a) RC-3rddh7 and HAZ-9fmhm4 carry no amendment for the changed verdict set.
disposition: fixed — dated amendment (6b4cbb5).

**finding-38**: record — (round 3a) receiver_events.rs:382-385 still says verdicts come only from `verify_envelope_against_chain`.
disposition: fixed (ac3c206).

**finding-39**: record — (round 3a) on-chain-client :51 and LLR-xv7auy :64 name REQ-2qa5r5 beside REQ-54txzh with no note marking it historical.
disposition: fixed — dated note at both lines (d8d3c0b).

**finding-40**: record — (round 3a) on-chain-client-hazards.md:1123 cites shifted `types.rs` lines.
disposition: fixed — re-resolved (d8d3c0b).

**finding-41**: record — (round 3a) on-chain-client-hazards.md:2016-2046 and chain-reading.md:19-22 assert the old "signing key" glossary state in the present tense.
disposition: fixed — dated notes; the terms sentence restructured (d8d3c0b).

**finding-42**: record — (round 3a) LLR-hezpr7's file never names SDD-f2s7bx.
disposition: fixed — SDD-f2s7bx heading (d8d3c0b).

**finding-43**: record — (round 3a) on-chain-client/docs/CONTEXT.md:15-17 root-term list omits Organisation public key.
disposition: fixed (d8d3c0b).

**finding-44**: requirement — (round 3a) "checked above this unit, by org-node" holds for state reads only; Event payloads carry `OrgPubKey` unchecked (latent, no consumer).
disposition: fixed — the boundary stated in REQ-54txzh's preamble, the risk file and the `types.rs` doc (d8d3c0b).

**finding-45**: record — (round 3a) on-chain/POST_POC.md:23-36 calls `orgPubKey` Ed25519.
disposition: fixed — dated note (d8d3c0b); `on-chain` is not a unit.

**finding-46**: record — (round 3a) app config `verify_commands` comment counts stale.
disposition: fixed — re-measured (6b4cbb5).

**finding-47**: code — (round 3a) whether a persistent `InvalidOrgPublicKey` stops the receiver loop is unpinned (suspected surviving mutation M5); also suspected survivor M8 (`map_state` argument swap behind feature `chain`).
disposition: fixed — tests through `outcomes_for_receive_error` for `InvalidOrgPublicKey`, `SeqNotEpoch`, `InviteOrgKeyMismatch` and `NoImportedInvite`; M5 killed (ac3c206). M8: the round-3 reviewer found it covered by `chain_state_is_read_into_typed_values` (not run by it).

### Round 3b — partial (org-members, root glossary, plans, app REQ-kn5rtx), read at d576dc7 (tree cb5258f), nothing run; a helper of the round-3 reviewer

**finding-48**: requirement — (round 3b) LLR-z954wj omits "bytes that do not decode as an Edwards point are refused"; no org-members test feeds an off-curve encoding (mutation P5/M1 survives).
disposition: fixed — clause added; off-curve case at all five entry points (6e34739).

**finding-49**: record — (round 3b) key-uniqueness.md:57-64's acceptance rests on group-key rules no item on this branch defines; the glossary says a PersonPublicKey comes from CGKA while org-node derives it from `member_seed`; "Member key and Person key never the same" unenforced.
disposition: fixed — the risk note says the rule is the owner's of 2026-10-04, recorded on the unmerged `worktree-person-requirements` branch, to be realised by the planned Member key rules change, and enforced until then by byte comparison within one encoding; glossary notes the per-persona seed; "never the same key" marked not yet enforced (6e34739).

**finding-50**: record — (round 3b) pre-switch ed25519 member keys: about half parse as X25519; the leaf hash domain and `canonical_bytes` carry no version tag, so an old record silently verifies with an unusable key (availability).
disposition: fixed — residual stated in the 2026-10-05 org-members risk file (6e34739).

**finding-51**: record — (round 3b) LLR-377ddr says pinned values; this change re-pinned them without amending it; soup.md's `person` row overstates.
disposition: fixed — dated amendment of LLR-377ddr citing the re-pin and its evidence; soup row corrected (6e34739).

**finding-52**: record — (round 3b) org-members' live REQs and RC use "Device key"; org-members/docs/CONTEXT.md has no mapping entry; LLR-z954wj says "device key set".
disposition: fixed — glossary entry as in org-node; LLR wording (6e34739).

**finding-53**: record — (round 3b) PR-b7khyw has no `resolution:` field, a 19-line resolution, and does not cite LLR-z954wj.
disposition: fixed — one-line `resolution:` citing LLR-z954wj and LLR-a645bx, detail below it (6e34739); line found missing again in round 3 (finding-90), fixed in 8aa8b74.

**finding-54**: record — (round 3b) stale names: architecture/README.md:60, README.md:16, design-derived.md:740-742, membership-hazards.md:479.
disposition: fixed — dated notes (6e34739).

**finding-55**: record — (round 3b) decomposition.md :50, :58, :132, :139: one amendment pasted four times, wrong for names; LLR-72p8bz mapping wrong; dated 2026-10-04; LLR-xyv6p9 unamended.
disposition: fixed — one correct note per item, LLR-xyv6p9 amended (6e34739).

**finding-56**: code — (round 3b) `DEVICE_EMPTY_SENTINEL` is implementor-chosen; nothing requires it to lie outside the DevicePublicKey space.
disposition: fixed for org-members — LLR-jfj6pc (the sentinel encodes no DevicePublicKey) and its tests (6e34739); the `person`-side constraint is a follow-up for `person` (Gaps).

**finding-57**: requirement — (round 3b) LLR-z954wj and LLR-a645bx say the public constructor reports `InvalidDeviceKey`/`InvalidPersonKey`; the constructors return `person::IdentityError`, org-members' variants arise only through `From`.
disposition: fixed — LLR wording names `IdentityError` and the `From` mapping; tests assert both (6e34739).

**finding-58**: code — (round 3b) `TryFrom` refusals checked `is_err` only; the postcard half cannot distinguish; LLR-z954wj misses order-8 points, y = p, y = p − 1; integration_test.rs:1861-1897 `is_err` only.
disposition: fixed — exact error assertions; the missing small-order and non-canonical cases added (6e34739).

**finding-59**: record — (round 3b) soup.md's ed25519-dalek row lists REQ-avmu3j and REQ-4umsuz though org-members/src no longer uses it.
disposition: fixed (6e34739).

### Merging master `1feb608` and `06c357b` — items the merge surfaced (author, not a review round; fixed with rounds 3a and 3b)

**finding-60**: requirement — (merge item m-1) org-members LLR-k6dhz7 (key-parse.md:11): master's `P2pMemberKey`/`P2pDeviceKey` parse accepting small-order keys with `InvalidKey`; contradicted (MISSING-TEST).
disposition: fixed — first superseded by a replacement under the owner's supersede-and-resolve ruling (6e34739); then, by owner answer Q4, LLR-k6dhz7 amended in place to the current rule and the replacement withdrawn (f35f9c4); risk note key-parse.md amended.

**finding-61**: requirement — (merge item m-2) LLR-t3p9zk names `P2pDeviceSlots`.
disposition: fixed — superseded by LLR-st6j2r (`person::DeviceSlots`); tests carry both IDs (6e34739).

**finding-62**: requirement — (merge item m-3) org-node LLR-mmdu38 (Edwards rule, `InvalidKey`, `From<&P2pMemberKey>`).
disposition: fixed — superseded at the merge (b77e125), then folded in place by owner answer Q4: LLR-mmdu38 states the X25519 rule, Debug and fail-closed cache clauses; replacement withdrawn (6324a1c, 53a3779).

**finding-63**: requirement — (merge item m-4) LLR-56hc77 (`MemberSeed::signing_keypair`).
disposition: fixed — member seed → `x25519_keypair`, device seed → `signing_keypair`; superseded (b77e125), then folded in place (Q4).

**finding-64**: requirement — (merge item m-5) LLR-bwb9pu omits `X25519Keypair` and `OrgPrivateKey`.
disposition: fixed — superseded to name them (b77e125), then folded in place (Q4); parent checked in round 3 (finding-83).

**finding-65**: requirement — (merge item m-6) LLR-8bum44 says "curve points" for pending-Invite keys.
disposition: fixed — superseded (b77e125); the trim withdrew the replacement and amended LLR-8bum44 in place (Invite blob keys parsed as PersonPublicKey and DevicePublicKey; master's import error, trim T4).

**finding-66**: requirement — (merge item m-7) LLR-ayrdr8's pinned bytes changed by this branch's two format changes.
disposition: fixed — new pinned values derived from master's bytes and the reason (b4fccf8); superseded at the merge, amended in place by the trim.

**finding-67**: record — (merge item m-8) master's amendment notes on SDD-rx2yvy and SDD-8cpyfa (curve points; "signed change").
disposition: fixed — dated notes (b77e125); both items amended in place by the trim.

**finding-68**: record — (merge item m-9) the amendments of SDD-sxp8hb, SDD-8cpyfa and SDD-rx2yvy add REQ-y7tsft and REQ-qn2erx.
disposition: fixed — this branch's replacement SDDs gained them (b77e125); moot after the trim, which withdrew those replacements and amended the master SDDs in place; REQ-qn2erx's "curve point" wording left to chain-authority (finding-83).

**finding-69**: record — (merge item m-10) PR-szkat6 (Organisation key conflation) fixed here by REQ-ech45n.
disposition: fixed — resolved at the merge (b77e125); the trim reopened it (T10), and round 4 restated its open part (finding-97). Net: open.

**finding-70**: record — (merge item m-11) PR-vkw22m (non-strict verify) is moot: nothing verifies a signature.
disposition: fixed — resolved; root cause and test `the_wire_form_has_no_signature_field` (b77e125, trim T10).

**finding-71**: record — (merge item m-12) PR-4b2v6p (role-typed keypairs) partly addressed.
disposition: no change to its status — dated note, stays open (b77e125).

**finding-72**: record — (merge item m-13) PR-hqwpg9's resolution names `MemberSeed::signing_keypair`.
disposition: fixed — dated note (b77e125).

**finding-73**: requirement — (merge item m-14) `OrgPrivateKey` has no LLR.
disposition: fixed — LLR-322xfu (b77e125); derived and assessed in round 3 (finding-83).

**finding-74**: record — (merge item m-15) this branch's architecture item for `keys.rs` claims `OrgPublicKey`, while master's SDD-swtd3w owns `types.rs`.
disposition: fixed — ownership reconciled (b77e125); after the trim withdrew that item, the owning SDDs trace REQ-ech45n and REQ-8jb4ny (finding-84).

**finding-75**: record — (merge item m-16) soup.md rows merged; the ed25519-dalek and serde rows name `sig_bytes`/`verify`.
disposition: fixed — rows corrected (b77e125).

### Round 3 — full, approve with findings; tree edab395 (HEAD ef6e261); reviewer saw org-members 241 passed, 1 ignored; on-chain-client 73; org-node 190; app 86 and vitest 30; every gate exit 0; 20 org-node mutations, 19 killed

Owner response (2026-10-05): the chain-authority rulings; option 1, the trim
(`docs/plans/2026-10-05-switch-trim.md`, answers Q1–Q4).

**finding-76**: code — (round 3) LLR-98ufry ("overwrites its 32 secret bytes with zeros when it is dropped") is unverified. Deleting `self.zeroize();` from `impl Drop for X25519Keypair` (`org-node/src/keys.rs:123`) leaves every org-node gate test green; `an_x25519_secret_is_zeroized_and_wiped_on_drop` only checks the `ZeroizeOnDrop` marker and calls `zeroize()` by hand.
disposition: no change to the code — no safe test observes memory after drop; recorded as verified by inspection: the test is renamed `an_x25519_key_pair_declares_zeroize_on_drop_and_zeroize_clears_its_secret`, and the LLR and robustness table say the Drop call is inspection-only (8aa8b74).

**finding-77**: requirement — (round 3) `receive_and_verify` and `receive_and_self_delete_if_revoked` read the chain for the unauthenticated message's `org_id` before the REQ-7h7qp3 sender check; SDD-zqc75b says "Before anything is decoded" though `recv_one` decodes the whole Wire message first.
disposition: moot — owner ruling (chain-authority): nothing about the sender is checked; the trim removed the sender check and withdrew SDD-zqc75b; the "before anything is decoded" wording corrected in the trim.

**finding-78**: requirement — (round 3) on a first admission nothing checks that the Invite's administrator keys appear in the verified record, so a forged Invite records the forger as administrator.
disposition: moot — owner: org-node has no administrator concept; the admin fields leave org-node in chain-authority's change 1; owner answer Q1 keeps `admin_member_key` from the Invite only as an unchecked record until then.

**finding-79**: requirement — (round 3) the app never runs REQ-ztdza4's post-verify sender check (it calls only `receive_and_self_delete_if_revoked`).
disposition: moot — REQ-ztdza4 amended to chain-authority's text: no sender check.

**finding-80**: requirement — (round 3) the amended REQ-kn5rtx rule contradicts itself (first-admission `UnknownSender` judged against the Invite filed as a verdict; the "local condition" exclusion dropped; "verification verdict" undefined in the app glossary).
disposition: fixed by the trim's T5 rewrite — verdicts are refusals judged against the chain or the node's record; the Invite-based verdicts are gone (54f0d8c, 9f2d1af); round 4 raised nothing on it.

**finding-81**: record — (round 3) `verifies: LLR-72p8bz` sits on the helper `fn encodes_no_device_public_key`, not on `hasher_domains_are_separated`.
disposition: fixed — moved to `hasher_domains_are_separated` (8aa8b74).

**finding-82**: record — (round 3) retired behaviour recorded as supersession, so tests carry `verifies:` for items they contradict (envelope_binding.rs:74, admission_sender.rs, service_stories.rs:453, verify_against_chain.rs:116/:306, newtypes.rs:340/:367).
disposition: fixed by the trim — owner's amend-in-place convention: the old items are amended to the current behaviour, the replacements withdrawn, and tests carry only IDs whose text they verify (T6, T7, T7b).

**finding-83**: requirement — (round 3) some parents do not state what the child claims: LLR-zyw5r2 satisfies REQ-y7tsft, which names neither the Organisation private key nor `X25519Keypair`; LLR-322xfu claims REQ-ech45n for Clone, not-Copy, no Display and redacted Debug, none of which REQ-ech45n states; REQ-qn2erx ("not a curve point") is reinterpreted in a child note instead of amended.
disposition: fixed — LLR-zyw5r2 folded into LLR-bwb9pu by Q4 and LLR-bwb9pu's parent checked and noted; LLR-322xfu marked derived and assessed (8aa8b74). REQ-qn2erx: deferred to chain-authority, which amends it.

**finding-84**: record — (round 3) REQ-ech45n and REQ-8jb4ny traced only by an SDD that owns `keys.rs` only; SDD-89es4z, SDD-af5vnt, SDD-swtd3w and SDD-pa6p7w own the code; LLR-sj7cd5, LLR-3fwykc and LLR-2dvhz8 under no owning SDD.
disposition: fixed — the owning SDDs trace REQ-ech45n and REQ-8jb4ny; the custody LLRs placed under their owning SDDs (8aa8b74).

**finding-85**: requirement — (round 3) LLR-z954wj's non-canonical-encoding clause cannot be observed separately: every non-canonical ed25519 encoding is off-curve, small-order or torsioned, so `is_canonical &&` in `person/src/device_key.rs:22-23` is an equivalent mutant.
disposition: fixed in the records — LLR-z954wj's note and PR-b7khyw say so, with the enumeration (8aa8b74); no test can isolate it (Gaps).

**finding-86**: requirement — (round 3) LLR-jfj6pc constrains only `Blake3Hasher`'s sentinel; the public `TrieHasher` lets any downstream hasher choose any sentinel; the org-members follow-up does not mention it.
disposition: fixed — noted in the org-members risk file's follow-up for `person` and org-members (8aa8b74); the constraint itself is open (Gaps).

**finding-87**: requirement — (round 3) the event-payload boundary is prose only; no requirement obliges a future consumer of `GenesisInitialized`/`RootUpdated` to parse `OrgPubKey`; `org-node/src/preflight.rs:114` reads state unparsed (diagnostic).
disposition: no change — owner ruling: on-chain-client stays byte-level and org-node parses; no consumer exists; the boundary is stated in three places. The preflight exception is noted in the risk file (8aa8b74).

**finding-88**: record — (round 3) superseded REQ-2qa5r5 named as current at chain-reading.md:48-58, hazards :1128, :1142, decomposition :544, :608, :767.
disposition: fixed — dated notes (8aa8b74).

**finding-89**: record — (round 3) glossary problems: root CONTEXT says a member is given the Organisation private key; org-node's Organisation secret entry cites PR-szkat6 as open/resolved inconsistently; Secret omits the Organisation private key; app CONTEXT lists "Device key" as a root term; avoided terms in new text; an unenforced rule stated without saying so; terms used in on-chain-client and app but defined only in org-node.
disposition: fixed — partly by the trim's T11 sweep (03f67f5), the rest in 8aa8b74.

**finding-90**: record — (round 3) previous dispositions that do not hold: PR-b7khyw has no `resolution:` line (round 2 finding-33, round 3b finding-53); round 2 finding-34's old test name still cited as current; LLR-e5c9ud's exclusion "not evidenced" though a test covers it.
disposition: fixed — `resolution:` line; current test names; LLR-e5c9ud evidenced (8aa8b74).

**finding-91**: record — (round 3) stale or incomplete statements (hazards.md:211, :266-272, :1177; org-node AGENTS.md:38, :40-48; soup.md:70 counts; app-hazards.md:1184-1188; org-members decomposition.md:472-474; delta.rs:66-67 "signer"; POST_POC.md's V2 `orgKeyType` list; the org-node plan lacks the epoch ruling).
disposition: fixed where still applicable after the trim (8aa8b74; the plan's dated note on the two later rulings).

### Round 4 — narrow (ef6e261..1fe6a12: the trim and round-3 leftovers), approve with findings; reviewer saw org-members 241 passed, 1 ignored; on-chain-client 73; org-node 183; app 84 and vitest 30; gates exit 0; 12 mutations, 11 killed; shared items byte-identical to chain-authority's 48969ab

**finding-92**: requirement — (round 4) any peer can pre-empt a node's pending first admission by relaying someone else's genuine admission: the node commits a record it is not in, consumes its Invite, and its own admission may then fail with `DeltaBaseMismatch` (reproduced). No risk file records it.
disposition: deferred — owner ruling 2026-10-05 ("ignore this for now, the other session is dealing with this"): to the chain-authority change, which reworks first admission; recorded here, not in this branch's risk files.

**finding-93**: code — (round 4) LLR-rys5nx's "Organisation public key from the chain" clause unverified (M12 survives: recording the Invite's `org_pub_key`).
disposition: fixed — `a_first_admission_whose_invite_names_another_organisation_key_is_committed` asserts the record holds the chain's Organisation public key and verifies LLR-rys5nx; M12 now fails it (95fcf59).

**finding-94**: record — (round 4) LLR-2dvhz8 re-trace leftovers: organisation_key.rs:160 verifies REQ-ech45n on a Debug test; unsigned-envelope.md:97-98 and type-safety.md:244 still place LLR-2dvhz8 under REQ-ech45n; the robustness table lacks LLR-2dvhz8 and LLR-3fwykc rows.
disposition: fixed — the Debug test verifies LLR-2dvhz8 alone; notes corrected; robustness rows added (95fcf59); LLR-2dvhz8 derived and assessed (1fe6a12).

**finding-95**: requirement — (round 4) no abnormal-case test on the no-Invite first-admission path.
disposition: fixed — `a_first_admission_with_no_imported_invite_that_misses_the_chain_root_commits_nothing`: refused with `RootMismatch`, store and Personas unchanged (REQ-xa6smf, LLR-mbjfq8) (95fcf59).

**finding-96**: requirement — (round 4) LLR-ctzkv7, LLR-g9vmbx and LLR-9zfnmb claim REQ-xa6smf/REQ-ztdza4 as parents, which no longer state them.
disposition: deferred — owner ruling 2026-10-05: re-trace left to chain-authority's change 1; a dated stale-parent note on each item (95fcf59).

**finding-97**: record — (round 4) PR-szkat6 reopened with its original defect text and `affects:` lines that no longer hold.
disposition: fixed — restated as its open part (the Organisation private key is not given to Members), `affects: REQ-ech45n, LLR-3fwykc`, dated (95fcf59).

**finding-98**: record — (round 4) comments at service.rs:1058, blobs.rs:12 and service.rs:1243 state removed checks.
disposition: fixed (95fcf59).

**finding-99**: record — (round 4) verify-and-commit.md:12-18's header differs from chain-authority's 48969ab (Device key); a later chain-authority merge will conflict there; this branch's text is correct.
disposition: no change — noted here for chain-authority (Gaps).

**finding-100**: code — (round 4, gate note) LLR-98ufry has no normal-case test tagged and "cannot be cloned" is untested; LLR-3fwykc and LLR-2dvhz8 missing from the robustness table.
disposition: fixed — `x25519_seed_round_trip_preserves_key` tagged as LLR-98ufry's normal case; the clone clause checked by a compile-time trait probe (`an_x25519_key_pair_cannot_be_cloned`); the Drop call stays inspection-only; table rows added (95fcf59).

**finding-101**: record — (final gate after merging master d8b9f9b) master's new app item LLR-7bk6qh (`app/docs/architecture/2026-10-05-decomposition.md`) lists `BadSignature` among the verdicts and `InvalidKey` among the receiver errors; on this branch `classify_receive_error` has neither and classifies `SeqNotEpoch` as a verdict and `InvalidOrgPublicKey` as a receiver error. Its tests already used the new set and passed.
disposition: fixed — LLR-7bk6qh amended in place with a dated note naming the variant set the classifier matches; the rule and its tests unchanged.

## Gaps

- on-chain-client's coverage floors sit on its documented accepted shortfall.
  org-node and the app have no `coverage_command` (pre-existing, recorded in
  their configs). Decision (branch) coverage is not measured for any unit this
  change touches.
- LLR-98ufry's Drop call (zeroize on drop) is verified by inspection only;
  mutation M19 survives by construction.
- LLR-z954wj's non-canonical clause is an equivalent mutant: the canonicity
  check in `DevicePublicKey::parse` cannot be told apart by any input.
- Intermittent `mbt_conformance` failure "Quint returned non-zero code" (11
  tests) seen once by the round-2 reviewer; not reproduced with
  `--no-fail-fast` or `--test-threads=1`, nor since.
- The shared cargo cache `/tmp/cargo_home_fuzz` was found corrupted on
  2026-10-06; the final gate ran with a scratch copy. Quint runs used scratch
  `QUINT_HOME` copies seeded from `~/.quint` (read-only by design).
- Follow-ups for `person` (merged, out of this change's scope):
  `DeviceTrieHasher`'s empty sentinel is unconstrained for hashers outside
  org-members (finding-56, finding-86); `to_fixed_slots` would drop a fifth
  device silently if the bound check were removed.
- Follow-ups for chain-authority: pending-admission pre-emption (finding-92);
  the stale parent links of LLR-ctzkv7, LLR-g9vmbx and LLR-9zfnmb
  (finding-96); the verify-and-commit.md header conflict with 48969ab
  (finding-99); REQ-qn2erx's "curve point" wording (finding-83); the in-place
  wording of master's signature LLRs (LLR-na7p4w, LLR-9fvb3y, LLR-e7s4ye,
  LLR-ybn5pr, LLR-cs4mpb, LLR-9sknpa, LLR-pzde8b, LLR-mcdh85, LLR-37cj3n) is
  this branch's (trim Q6), for chain-authority to review.
- In the interim until chain-authority's change 2, any peer relaying a genuine
  admission or update can substitute the Organisation secret (PR-ve9zw8, owner
  answer Q2).
- The round-4 fixes (95fcf59) were not put to a further reviewer.

## For the commit message

Implements (new): REQ-54txzh, LLR-hezpr7 (on-chain-client); LLR-z954wj,
LLR-a645bx, LLR-st6j2r, LLR-jfj6pc (org-members); REQ-txvtm9, REQ-8jb4ny,
REQ-ech45n, RC-95dgg8, LLR-3jjgtw, LLR-98ufry, LLR-9f5hmr, LLR-322xfu,
LLR-rys5nx, LLR-sj7cd5, LLR-3fwykc, LLR-2dvhz8 (org-node).

Implements (amended in place, 2026-10-05): org-node REQ-ag6kqm, REQ-nhe2zu,
REQ-xa6smf, REQ-ztdza4, REQ-hzm4kt, RC-pm9kmx, RC-b6mydy, SDD-sxp8hb,
SDD-kk2y3e, SDD-na9nc3, SDD-rx2yvy, SDD-8cpyfa, SDD-72ddm6, SDD-z85ux9,
LLR-e58j8m, LLR-ctzkv7, LLR-na7p4w, LLR-9fvb3y, LLR-e7s4ye, LLR-p8uu47,
LLR-ybn5pr, LLR-cs4mpb, LLR-9sknpa, LLR-pzde8b, LLR-mcdh85, LLR-g9vmbx,
LLR-rv4vux, LLR-s7yu4k, LLR-rb8r65, LLR-ghja3x, LLR-vdyu65, LLR-zj88e6,
LLR-9zfnmb, LLR-j83kc8, LLR-u6rq4s, LLR-37cj3n, LLR-mbjfq8, LLR-y2v8v2,
LLR-xq9nrq, LLR-e5c9ud, LLR-3q63zv, LLR-6dc598, LLR-tax3pm, LLR-8hdu9x,
LLR-ayrdr8, LLR-mmdu38, LLR-56hc77, LLR-bwb9pu, LLR-8bum44; org-members
LLR-k6dhz7, LLR-377ddr, LLR-pys2ek, LLR-w5nkbu, LLR-xyv6p9, LLR-68tka5,
LLR-72p8bz, LLR-kdhd2v, LLR-sa3ugj; app REQ-kn5rtx, RC-3rddh7.

Superseded by this change: REQ-2qa5r5 by REQ-54txzh, LLR-nq7nhg by LLR-hezpr7
(on-chain-client); LLR-t3p9zk by LLR-st6j2r (org-members).

Opens: PR-ve9zw8
Resolves: PR-b7khyw, PR-u4c2vp, PR-vkw22m
(PR-szkat6: resolved, then reopened in this change; net still open.)
