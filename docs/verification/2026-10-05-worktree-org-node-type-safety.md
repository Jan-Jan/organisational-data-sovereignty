# Verification — org-node parses at the system's edge (2026-10-05)

branch: worktree-org-node-type-safety
reviewer: eight rounds of independent subagent review at `merge-change` step 6a, one fresh reviewer per round, each given the diff, the ledgers, the ADR and the plan and no author narrative, each re-running the org-members, org-node and app `verify_commands` in its own nested worktree (round 8's first dispatch was cut off by an expired login and re-dispatched in full). The owner chose to continue after every round that raised a `code` or `requirement` finding, and ruled to close review after round 8 once its findings were fixed.
verdict: merge. Every round's reviewer found that the code meets the requirements it claims; the code defects review found — a chain-read cache that kept serving a superseded root after a refused parse (round 3), and untested or unowned behaviour (rounds 1, 2, 7, 8) — were fixed under TDD. The remaining findings were requirement scope (REQ-qn2erx widened to keys, then narrowed to the curve-point parse), conflicts brought in by two base merges (`person`'s X25519 ruling; org-node's architecture ledger), and record accuracy. All 34 findings are dispositioned below.
reproduced: yes for PR-hqwpg9 — `records_and_wire_messages_never_render_secret_bytes` was run against the pre-change records (`[u8; 32]` fields) and failed at runtime with the member seed and Organisation secret rendered ("208,209,210", "64,65,66"), then passed after the change. The round-3 cache defect was reproduced by `refresh_refused_at_parse_clears_the_cached_state` failing against the old early return. Every other new test was watched red before it passed, by the absent implementation or by a named mutation reverted before commit; exceptions are stated in the red → green table.

Change: org-node parses at the system's edge — secrets (member seed, device seed, Organisation secret, store key) in redacted types; `OrgPublicKey`, `ChainAccount`, `PersonaId`, `Epoch`, `SequenceNumber`; typed Persona store, Join request, Invite and record snapshot, refused with the field named when a value fails its parse (REQ-y7tsft, REQ-qn2erx); org-members' key types gain `parse`/`TryFrom<[u8; 32]>` and `P2pDeviceSlots::new` becomes `parse`; the app parses user input at its edge. Stored, wire, blob and calldata bytes unchanged (pinned before the refactor). Branched from `master` at `24317e3`; local master merged in at `f688a3c` (person unit) and `05f6f04` (org-node architecture, ratchet tooth 4).
Plan: `docs/plans/2026-10-04-org-node-type-safety.md`.

Units touched: `org-node` (code, tests, ledgers, SOUP, AGENTS.md, config), `org-members` (key and device-set constructors, tests, ledgers, AGENTS.md, one Quint comment) and `app` (edge parsing, tests, config comment). Impact set, `check-units.sh --impact master..HEAD`: `org-members` touched, `org-node` touched, `app` touched.

Base: `git fetch origin` succeeded each round; `origin/master` (`24317e3`) is an ancestor of local `master`, which this repository integrates locally, so local `master` was merged each time it moved. At the `05f6f04` merge the owner ruled to reconcile the designs: this change's two design items (never on master) were withdrawn and its LLRs re-homed under org-node's decomposition, nine master items amended in place, and org-node's `problem_open_max` raised from 10 to 15.

Problem-ledger delta: resolves PR-hqwpg9 (org-node). Opens PR-szkat6 (Envelope signing key conflated with the Organisation public key), PR-vkw22m (non-strict Envelope signature check; narrowed and moved to org-node after PR-b7khyw reached master), PR-4b2v6p (signing key pairs carry no role), PR-k2xxaq (chain reader keeps a superseded state after a failed fetch) — all org-node, all deferred by owner ruling. Amends master's LLR-e58j8m, LLR-rc74nq, LLR-rm9x4z and SDD-swtd3w, SDD-sxp8hb, SDD-pa6p7w, SDD-af5vnt, SDD-vee2fq, SDD-z85ux9, SDD-msb6xh, SDD-8cpyfa, SDD-89es4z, SDD-rx2yvy, SDD-kwncn7 in place.

## The gate

Final run (gate 10) on tree `e6fdf6d5735478312ffa0e6554df0d4d033756c8` (HEAD `63962f7`), the tree after the last fix and before this record.

| Gate | Result |
| --- | --- |
| `check-units.sh` | exit 0 — `units: 5, disclaimed 6; tracked paths 531` (this record makes 532) |
| org-members `cargo test -p org-members` | **225 passed, 0 failed**, 1 ignored by design (`preflight_probe`) — integration 161, mbt_conformance 32, newtypes 20, fuzz 9, encoding_golden 3 |
| org-members quint | typecheck of `membership_types`, `membership`, `membership_mbt` exit 0; `quint test membership.qnt` **63 passing**; `quint run membership_mbt --invariant=mbtInv` no violation |
| org-members `make coverage-org-members` | lines **94.91%** (floor 92), regions **93.86%** (floor 91), functions 91.78%; decision coverage unmeasured |
| org-node `cargo test` (features `app,test-support`, 20 harnessed targets) | **167 passed, 0 failed** — admission_sender 39, verify_against_chain 19, persona_records 18, node_value_types 11, chain_write_pure 8, envelope_binding 8, service_lifecycle 8, store_at_rest 7, key_custody 6, secret_redaction 6, transport_handshake 6, value_types 6, blob_exchange 5, encoding_golden 5, wire_frame_bound 5, chain_read_state 3, service_stories 3, calldata_typed 2, lib 1, transport_networked 1 |
| org-node bolero targets (1 s budget each) | no failure — `fuzz_envelope_decode` 162,300 iterations, `fuzz_first_admission_base` 2,690, `fuzz_verify_against_chain` 845 |
| org-node quint | typecheck `protocol`, `ods_instances` exit 0; `forkSafety`, `revocationSafety`, `revokedExcludedFromOrgSecret`, `tauWindow`, `convergence` no violation |
| app cargo, 6 targets | **82 passed, 0 failed** — receiver_events 28, org_id_parsing 14, startup_policy 12, ipc 12, connection_status 10, receiver_guard 6 |
| app `npm run check` / `npm run test` | 194 files, **0 errors**, 1 pre-existing warning (`@types/node`) / vitest **30 passed** |
| `check-ids.sh` (bare, per unit) | exit 0, silent, in org-members, org-node, app |
| `check-trace.sh` | exit 0 in all three; org-members `REQ 14, HAZ 8, RC 10, SDD 6, LLR 47, PR 8`, open 2; org-node `REQ 22, HAZ 8, RC 11, SDD 19, LLR 121, PR 17`, open 13 of 15, oldest 26 days of 30; app `REQ 22, HAZ 6, RC 12, PR 6`, open 5 |
| Implements map / robustness | all 16 IDs traced: 12 LLRs with `verifies:` tests, the two REQs and two RCs through them; no MISSING-TEST in any unit. Normal and abnormal-input tests for every LLR with a refusal path; LLR-sz4xhc, LLR-bwb9pu, LLR-56hc77, LLR-s7whrn and LLR-scgk5j accept every input and are tested at boundary values instead |
| Working tree | clean but for this record, uncommitted at the time |

Environment: `CARGO_HOME=/tmp/cargo_home_fuzz` (`~/.cargo` is read-only); `QUINT_HOME=<worktree>/target/quint_home` with the evaluator copied read-only from `~/.quint`.

**Coverage, gap accepted.** org-members meets its line and region floors; decision coverage is unmeasured, and org-node and app have no `coverage_command` — the class C gaps recorded in their configs. The owner explicitly accepted these gaps for this merge on 2026-10-04 (verify-before-merge check 5).

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| `LLR-ayrdr8` | `persona_store_plaintext_is_pinned` | constants regenerated from the pre-refactor code byte for byte; then `PendingInvite` key fields swapped → round trip passed, field read failed; reverted |
| `LLR-ayrdr8` | `genesis_and_update_calldata_is_pinned` | epoch written `to_le_bytes` → update calldata failed; reverted |
| `LLR-ayrdr8` | `admission_wire_message_is_pinned` | `org_secret`/`genesis_snapshot` swapped in `WireMessage` → `Err(Malformed)`; reverted |
| `LLR-ayrdr8` | `invite_and_join_request_text_is_pinned` | `name`/`surname` swapped in `JoinRequest` → field read failed ("Jones" vs "Bob"); reverted |
| `LLR-ayrdr8` | `truncated_pinned_values_are_refused` | `decode_body` padding and retrying → truncated body accepted; reverted |
| `LLR-ayrdr8` | `typed_update_calldata_is_the_pinned_calldata`, `the_revive_runtime_call_carries_the_pinned_calldata` | E0432 before `update_calldata` existed; root/key swap inside the wrapper → both failed; reverted |
| `LLR-k6dhz7` | `key_parse_accepts_exactly_what_decoding_accepts` | compile-red before `parse`/`TryFrom` existed; `parse` always `Err(InvalidKey)` → failed; reverted |
| `LLR-k6dhz7` | `key_parse_refuses_bytes_off_the_curve` | compile-red only (no `InvalidKey`, no `parse`) |
| `LLR-t3p9zk` | `device_slots_parse_holds_keys_sorted` | compile-red only (no `P2pDeviceSlots::parse`) |
| `LLR-t3p9zk` | `device_slots_parse_refuses_too_many_and_repeated_keys` | compile-red; `MAX_DEVICES` check removed → failed; reverted |
| `LLR-sz4xhc` | `secret_types_redact_debug_and_give_bytes_only_through_the_accessor` | E0432 before the types existed; Debug printing the bytes → failed; added `Display` → failed; reverted |
| `LLR-sz4xhc` | `secret_debug_is_the_same_whatever_the_bytes` | Debug printing the bytes → failed; reverted |
| `LLR-sz4xhc` | `secret_types_serialise_as_the_plain_bytes` | **compile-red only** — no runtime mutation tried (derived transparent serde) |
| `LLR-56hc77` | `a_seed_yields_its_key_pair_and_a_key_pair_its_seed`, `seeds_at_the_byte_bounds_yield_working_key_pairs` | E0599 before `signing_keypair` existed; a flipped seed bit → both failed; reverted |
| `LLR-mmdu38` | `org_public_key_accepts_every_curve_point_unchanged`, `org_public_key_refuses_bytes_off_the_curve` | E0432; `parse` decompressing a fixed array → both failed; reverted |
| `LLR-mmdu38` | `chain_state_is_read_into_typed_values`, `chain_state_with_an_off_curve_key_is_refused_as_invalid_key` | E0432 before `org_state_from_chain`; parsing `[0u8; 32]` → both failed; reverted |
| `LLR-mmdu38` | `refresh_refused_at_parse_clears_the_cached_state` | failed against the old early return (epoch-1 state still served); fixed |
| `LLR-mmdu38` | `org_public_key_debug_is_its_name_and_first_four_bytes` | impl replaced by `derive(Debug)` → failed; restored (its "no full hex" assert is a backstop, never red alone) |
| `LLR-s7whrn` | `tag_types_hold_their_value_unchanged`, `tag_types_accept_every_boundary_value_and_serialise_as_it` | E0432; `Epoch::new` storing value+1 → both failed; reverted |
| `LLR-s7whrn` | `tag_types_debug_renders_their_value` | `derive(Debug)` on `ChainAccount`, bare-value impls on `PersonaId`, `Epoch`, `SequenceNumber` → failed for each; restored |
| `LLR-bwb9pu`, `REQ-y7tsft`, `RC-8a4xjb` | `records_and_wire_messages_never_render_secret_bytes` | **PR-hqwpg9 reproduction:** run against the pre-change records → failed at runtime rendering the seed and Organisation secret; green after the change |
| `LLR-bwb9pu` | `secrets_at_the_high_byte_bound_and_many_records_stay_unrendered` | failed against the pre-change records ("255,254,253"); green after |
| `LLR-bwb9pu` | `a_signing_key_pair_never_renders_its_seed` | temporary Debug printing the seed → failed; reverted |
| `LLR-scgk5j` | `the_store_key_renders_as_its_marker`, `the_store_key_renders_the_same_for_every_passphrase` | **compile-red only** (E0425, seam missing); the runtime mutation was refused by the sandbox as security-weakening and not worked around |
| `LLR-scgk5j` | `the_store_key_has_no_display_and_is_not_copy` | E0425; temporary `Copy` → failed; temporary `Display` → failed; reverted |
| `LLR-g76zqd`, `REQ-qn2erx`, `RC-zutc67` | `create_persona_holds_the_parsed_details_across_a_reopen` | compile-red (17 errors: untyped API, no seam); green after |
| `LLR-g76zqd` | `a_persona_record_decoded_directly_refuses_an_invalid_handle` | handle lowercased before parse → failed; reverted |
| `LLR-q6n25z` | `persona_details_parse_into_their_types_and_create_a_persona`, `persona_details_refuse_each_invalid_field_naming_it` | E0432 before `PersonaDetails` existed; green after |
| `LLR-q6n25z` | `non_nfc_persona_details_are_stored_and_exported_in_nfc` | `to_nfc` made the identity → failed; reverted |
| `LLR-q6n25z` | `persona_details_with_two_invalid_fields_report_the_first` | reversed parse order → failed; name/surname swapped → failed; reverted |
| `LLR-8bum44` | `a_store_of_valid_records_opens_with_every_field_parsed`, `a_valid_join_request_imports_with_every_field_parsed` | compile-red (untyped records); green after |
| `LLR-8bum44` | `a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field` | one field mislabelled → failed naming the mislabel; reverted |
| `LLR-8bum44` | `a_store_with_every_record_kind_opens_with_every_field_parsed` | `admin_member_key` parsed from the `org_pub_key` bytes → failed; reverted |
| `LLR-8bum44` | `a_join_request_holding_an_invalid_value_is_refused_naming_the_field` | `parse_field` ignoring its field → failed; reverted |
| `LLR-8bum44` | `a_record_snapshot_holding_an_invalid_value_is_refused_naming_the_field` | snapshot decoded directly as typed → `Chain(..)` not `InvalidField`; reverted |
| `LLR-8bum44` | `a_valid_record_snapshot_decodes_with_every_field_parsed` | device keys dropped in the parse → failed; reverted |
| `LLR-8bum44` | `a_valid_invite_imports_as_a_pending_invite` | pending-invite push dropped → failed; reverted |
| `LLR-8bum44` | `an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored` | saving before decoding → "nothing stored" failed; the refusal itself is structural (no off-curve typed key can be built), not mutable |
| `LLR-8bum44` | `a_record_snapshot_with_an_invalid_handle_and_member_key_reports_the_handle`, `a_store_with_two_invalid_fields_in_one_record_reports_the_first_in_record_order` | key parsed before handle / `admin_member_key` first → failed; reverted |
| `LLR-8bum44` | `a_non_nfc_join_request_imports_with_its_details_in_nfc` | `to_nfc` made the identity → failed; reverted |

## What was wrong, and what was built

PR-hqwpg9: `PersonaRecord`, `OrgRecord`, `StoreData` and `WireMessage` derived `Debug` over plain `[u8; 32]` seeds and the Organisation secret, and the store key was a plain array, so any `{:?}` of them wrote secret key material in clear. Measured by `records_and_wire_messages_never_render_secret_bytes` failing against the pre-change code. Built: `MemberSeed`, `DeviceSeed`, `OrgSecret` and the store-private `StoreKey`, each with a redacted `Debug`, no `Display`, no `Copy`, bytes only through `expose_secret` or serialisation (which writes the plain 32 bytes, so stored and wire bytes are unchanged); seeds become key pairs only through their own type (`from_seed`/`to_seed` removed).

The rest of org-node's plain values took types: `OrgPublicKey` (curve-point parse — interim, see Gaps), `ChainAccount`, `PersonaId`, `Epoch`, `SequenceNumber`; the store, Join request, Invite and record snapshot decode through private `Raw…` mirrors and refuse a value that fails its parse with `OrgNodeError::InvalidField { field, .. }`; `PersonaDetails::parse` refuses invalid persona details at creation, on the member's own device. A chain state whose key fails its parse now clears the reader's cache rather than leaving a superseded root in it. org-members' key types gain `parse`/`TryFrom<[u8; 32]>` accepting exactly what deserialisation accepted; `P2pDeviceSlots::new` is renamed `parse`. The app parses at its edge. Every stored, wire, blob and calldata byte is pinned by `encoding_golden.rs`, written before the refactor and unchanged since its first commit; a reviewer also ran it against the pre-refactor source (5/5).

## Review

### Round 1

**finding-1**: requirement — REQ-qn2erx's "refuse to create a Persona … reporting the field" was carried out only by app code (commands.rs:96-98) with no traced test; org-node's create_persona takes parsed values; the annotated test exercised only org-members' parse. (round 1, finding 1)
disposition: new LLR-q6n25z under SDD-af5vnt and `PersonaDetails::parse` in org-node, used by the app; `persona_details_refuse_each_invalid_field_naming_it` reddens without it (compile-red, then mutation).

**finding-2**: requirement — the record-snapshot refusal in first_admission_base (InvalidField member.*) and the Invite off-curve refusal had no LLR and no test. (round 1, finding 2)
disposition: LLR-8bum44 amended to cover the snapshot decode and Invite import; `a_record_snapshot_holding_an_invalid_value_is_refused_naming_the_field` and `an_invite_holding_a_key_that_is_not_a_curve_point_is_refused_and_nothing_stored` added, each red under a named mutation.

**finding-3**: code — store-open robustness tested only 2 of the 13 field names LLR-8bum44's design promises. (round 1, finding 3)
disposition: `a_store_with_any_one_field_invalid_is_refused_naming_exactly_that_field` covers all 13 fields; red under a mislabelled field.

**finding-4**: code — LLR-ayrdr8's calldata pin exercised only the untyped build_update_calldata, not the rewritten typed wrappers. (round 1, finding 4)
disposition: `update_calldata` extracted; `calldata_typed.rs` pins the typed wrappers to the golden constants; red under a root/key swap.

**finding-5**: record — the LLR-56hc77 assessment claimed it removes the member-seed-as-device-seed route; SigningKeypair is role-less, so a wrong-role key pair still passes. (round 1, finding 5)
disposition: LLR-56hc77 assessment corrected; wrong-role key pairs recorded as residual and filed as PR-4b2v6p (owner ruling: follow-up).

**finding-6**: record — SigningKeypair's Debug relies on ed25519-dalek 2.2.0 omitting the secret, unrecorded and untested. (round 1, finding 6)
disposition: `SigningKeypair` added to LLR-bwb9pu; `a_signing_key_pair_never_renders_its_seed` red under a Debug printing the seed; SOUP row records the reliance.

**finding-7**: record — the "Interface" list of observable changes omitted most public signature changes. (round 1, finding 7)
disposition: Interface list rewritten from the diff of pub items.

### Round 2

**finding-8**: record — Persona details are canonicalised to NFC at creation (`PersonaDetails::parse`), changing what is stored, listed and exported in a Join request for non-NFC input; not recorded as an observable change and not traced. (round 2, finding 1)
disposition: LLR-q6n25z amended (stored in NFC); observable change recorded and assessed; `non_nfc_persona_details_are_stored_and_exported_in_nfc` red under an identity `to_nfc`.

**finding-9**: code — no test made two fields invalid at once, so the parse order LLR-q6n25z states and the record-order change recorded for snapshots were unchecked. (round 2, finding 2)
disposition: two-invalid-field tests for persona details, snapshot and store; each red under a reordered parse.

**finding-10**: code — LLR-scgk5j's "no Display / not Copy" clauses were checked only by an unannotated unit test in `store.rs`, outside the gate's test paths. (round 2, finding 3)
disposition: `store_key_traits_for_test` seam and `the_store_key_has_no_display_and_is_not_copy`; red under a temporary `Copy` and a temporary `Display`.

**finding-11**: record — the plan named draft ledger paths that no longer exist. (round 2, finding 4)
disposition: plan paths repointed to the dated files.

### Round 3

**finding-12**: requirement — LLR-8bum44's key refusals (org, member, pending-invite, join-request keys; Invite import) satisfied no requirement: REQ-qn2erx named only handle, name and surname. (round 3, finding 1)
disposition: owner ruling: REQ-qn2erx widened to keys in place (an Invite's refusal reports the Invite, not the field); HAZ-vfjy32, RC-zutc67 and the assessment amended.

**finding-13**: code — `OnChainReader::refresh` returned early on an off-curve key and kept the previous state cached, so `get_org_state` served a superseded root after a refused refresh. (round 3, finding 2)
disposition: owner ruling: fail closed. `OrgStateCache::store_fetched` clears the cache on a refused parse; LLR-mmdu38 amended; `refresh_refused_at_parse_clears_the_cached_state` red against the old early return. The pre-existing staleness after a failed fetch is PR-k2xxaq (owner ruling: separate change).

**finding-14**: record — Join-request import also canonicalises to NFC; not recorded. (round 3, finding 3)
disposition: observable change extended; `a_non_nfc_join_request_imports_with_its_details_in_nfc` red under an identity `to_nfc`.

**finding-15**: record — the verify-line comments in org-node and app config and the plan's Verification header did not record the new targets and counts. (round 3, finding 4)
disposition: config comments and the plan's Verification header record the targets and measured counts.

### Round 4

**finding-16**: requirement — REQ-qn2erx said a store or snapshot holding a key "the member-record rules … would refuse" is refused, but a member snapshot's device key set is not parsed as `P2pDeviceSlots`, so a repeated key or more than four opens and is refused later by org-members (`Trie(..)`). (round 4, finding 1)
disposition: owner ruling: narrow. REQ-qn2erx, LLR-8bum44 and the assessment amended: the key clause is each key's curve-point parse; the device-set rules stay with org-members at rebuild.

**finding-17**: record — the plan's Implements header omitted LLR-q6n25z. (round 4, finding 2)
disposition: Implements header completed.

### Round 5

**finding-18**: requirement — the master merge brought the owner's ruling (recorded in `person`, REQ-7gz72r) that the Organisation public key is an X25519 key checked by `person`'s rule; this change names `OrgPublicKey` for that role but checks an ed25519 Edwards point, and recorded no conflict. (round 5, finding 1)
disposition: recorded, not changed (owner ruling): the Edwards rule is interim for the key the field holds today; LLR-mmdu38, the glossary and PR-szkat6 state that resolving PR-szkat6 must move `OrgPublicKey` to `person`'s X25519 rule.

**finding-19**: record — PR-vkw22m and the SOUP row said the non-strict `verify` accepts non-canonical signatures; in ed25519-dalek 2.2.0 it differs from `verify_strict` only on a small-order public key and a small-order `R`. (round 5, finding 2)
disposition: PR-vkw22m and the SOUP row corrected to "a small-order public key and a small-order R".

**finding-20**: record — PR-4b2v6p listed `affects: REQ-y7tsft`, which a key-pair role swap does not affect. (round 5, finding 3)
disposition: PR-4b2v6p affects LLR-56hc77 only.

### Round 6

**finding-21**: record — the risk file judged the S3/P1 residuals of HAZ-uy8sxm and HAZ-vfjy32 "acceptable", contradicting the unit's matrix and register (every S3 residual not acceptable; overall UNACCEPTABLE). (round 6, finding 1)
disposition: owner ruling: both residuals restated S3/P1, not acceptable; rows added to the register's residual table.

**finding-22**: record — the app moved org-members into its production dependencies for `MemberId` and `RootHash` without a declared `depends_on` edge. (round 6, finding 2)
disposition: owner ruling: org-node re-exports `MemberId` and `RootHash`; the app imports them from `org_node`; `app/src-tauri/Cargo.toml` identical to master.

### Round 7

**finding-23**: requirement — master's LLR-rm9x4z said `get_org_state` returns `None` only for an Organisation with no slot; under LLR-mmdu38 (fail closed) it returns `Ok(None)` after a refused parse. (round 7, finding 1)
disposition: LLR-rm9x4z amended in place to state the fail-closed exception; cross-reference from LLR-mmdu38.

**finding-24**: record — LLR-8bum44, LLR-g76zqd (and, found while fixing, LLR-bwb9pu) sat under items that do not own all the code they constrain; the SDD-pa6p7w section claimed `types.rs`. (round 7, finding 2)
disposition: LLR-mmdu38 moved to SDD-swtd3w, LLR-8bum44 to SDD-af5vnt; "also constrained by" notes and `traces:` additions on every other owning item; `first_admission_base` assigned to SDD-8cpyfa.

**finding-25**: record — `chain_write::calldata::update_calldata` was owned by no item. (round 7, finding 3)
disposition: SDD-msb6xh names `update_calldata`.

**finding-26**: requirement — LLR-sz4xhc said secret bytes leave only through `expose_secret`; they also serialise as the plain bytes (owner ruling), which a test verified without the LLR stating it. (round 7, finding 4)
disposition: LLR-sz4xhc amended: bytes leave only through `expose_secret` or serialisation (owner ruling), which `secret_types_serialise_as_the_plain_bytes` verifies.

**finding-27**: record — a measurement note misattributed the service.rs line drop to removing a test module master had already relocated. (round 7, finding 5)
disposition: attribution corrected.

**finding-28**: record — three stale SOUP statements (an envelope.rs cite, "the one hand-maintained decode invariant", "fifty Chain(String) sites"). (round 7, finding 6)
disposition: the three SOUP statements re-measured and corrected.

**finding-29**: record — service.rs line cites in the decomposition and five open problem reports shifted by this change. (round 7, finding 7)
disposition: every stale cite re-measured by script against master and HEAD and corrected (decomposition, risk files, six problem reports).

**finding-30**: record — the Interface list omitted `test_support::build_dispatch_tx`'s signature change. (round 7, finding 8)
disposition: Interface list re-derived by script; `build_dispatch_tx` and four probe traits added.

**finding-31**: record — the change in Debug rendering of records (personal fields redacted, keys shortened) was not recorded. (round 7, finding 9)
disposition: "Debug rendering" observable change recorded and assessed.

### Round 8

**finding-32**: requirement — the hand-written Debug of `OrgPublicKey` and `ChainAccount` was stated by no LLR and checked by no test. (round 8, finding 1)
disposition: Debug clauses added to LLR-mmdu38 and LLR-s7whrn; `org_public_key_debug_is_its_name_and_first_four_bytes` and `tag_types_debug_renders_their_value` red under `derive(Debug)` / bare-value impls.

**finding-33**: record — two stale comments (`membership.qnt` naming `P2pDeviceSlots::new`; `verify_against_chain.rs` naming the removed `ADMIN_DEVICE_SEED`). (round 8, finding 2)
disposition: both comments corrected.

**finding-34**: record — `node_value_types.rs`'s header placed LLR-mmdu38 under SDD-pa6p7w. (round 8, finding 3)
disposition: the test header and org-node/AGENTS.md placement sentence corrected.

## Gaps

- **Coverage.** Decision coverage is unmeasured in org-members, and org-node and app have no coverage command (owner-accepted for this merge).
- **Residual risk not acceptable.** HAZ-uy8sxm and HAZ-vfjy32 are S3/P1 with residuals not acceptable under the unit's matrix, in line with org-node's register; the controls stop where the owner ruled: no zeroise, `expose_secret` reachable, no migration for stores written before this change.
- **`OrgPublicKey`'s rule is interim.** It checks an ed25519 Edwards point, right for the key the field holds today (the administrator's Member-as-a-group key) and wrong for the intended X25519 Organisation key; resolving PR-szkat6 must move it to `person`'s X25519 rule.
- **Weak and non-canonical keys** are still accepted by org-members' key types (PR-b7khyw, on master) and Envelope signatures are checked with the non-strict `verify` (PR-vkw22m).
- **Device-key set rules** are not applied when a record is loaded; org-members refuses them when the members are rebuilt (owner ruling narrowing REQ-qn2erx).
- **Compile-red only:** `secret_types_serialise_as_the_plain_bytes`, `key_parse_refuses_bytes_off_the_curve`, `device_slots_parse_holds_keys_sorted`, and the two `StoreKey` rendering tests (the sandbox refused the security-weakening mutation; not worked around).
- **Fuzzing depth.** Master's rewritten `fuzz_first_admission_base` and `fuzz_verify_against_chain` run only about 2,700 and 850 iterations in their 1-second budget, against about 137,000 and 200,000 before; they pass, but the evidence they add is thin at that rate.
- **Problem limits.** org-node holds 13 open reports against a raised limit of 15, and four of them (PR-vt244s, PR-2dmjzj, PR-u4c2vp, PR-w88sr9, not this change's) reach the 30-day age limit around 2026-10-09.
- **Not covered by the final review:** this record itself, and the fixes to round 8's findings, which were gated but not reviewed again (owner ruling).
