# Verification — person unit and its identity types (2026-10-05)

One record per change, written at `merge-change` step 6b and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line.

branch: worktree-person-unit
reviewer: seven independent agent reviewers, one fresh subagent per round (rounds 1–7), each with no implementation narrative, each running the suite in its own nested review worktree
verdict: approve with findings in every round; round 7 raised no code defect in the library, and its findings were closed under the owner's bounded-stop rule (below) without an eighth round
reproduced: not applicable as a defect fix — this change builds a new unit; the one defect it records (PR-b7khyw, org-members' false SOUP claim about ed25519-dalek) was reproduced by the characterisation test `a_small_order_point_dalek_accepts_is_rejected`, which first asserts that `VerifyingKey::from_bytes` accepts the identity point

Change: registers the `person` unit (class C) and builds the identity types
org-members is to use — `Name`, `Surname`, `DevicePublicKey`,
`PersonPublicKey`, `DeviceSlots`, the device sub-trie and the exported X25519
validity check. Branched from `master` at `24317e3`; master was merged at
every round (local `master` and `origin/master` both `24317e3` throughout).
Plan: `docs/plans/2026-10-04-person-shared-types.md` (scope, dependency
assessment, owner rulings) and
`docs/plans/2026-10-04-person-shared-types-implementation.md` (tasks; this
change is its person-only part).

Units touched: person, org-members (one problem report only), plus the root
`.guardrails/units.yaml`, `Cargo.toml`, `Makefile` and `.github/workflows/rust.yml`.
Impact set (`check-units.sh --impact master..HEAD`): org-members, on-chain-client,
org-node, app, person — all five, because the root manifest changed.

## The gate

Final gate: run on HEAD `2f1c670`, tree `61f08c6a0b7eacdefaee909ecdc7ebe6d6b46763`, over
the whole impact set (log `worktree-person-unit-gate-final.log` in the session
scratchpad; 45 commands, all exit 0, verdicts read from counts). Seven earlier
gate runs, one per review round, were green on the trees they measured.

| Gate | Result |
| --- | --- |
| person `cargo test -p person` | 73 passed, 0 failed (also with `--locked`; Cargo.lock unchanged, lockfile version 3) |
| person `cargo clippy -p person --all-targets -- -D warnings` | clean; also clean with `--no-default-features`; `cargo fmt --check` clean |
| person wasm32 `cargo check` | passes with and without default features |
| person `make coverage-person` | lines 215/215, regions 331/331 (100%), floors 99/99; nightly branches 28/28 (measured during review, Makefile record) |
| org-members `verify_commands` | 221 passed, 1 ignored (`preflight_probe`, by design); quint: 3 typechecks, 63 passing, `mbtInv` no violation |
| org-members `make coverage-org-members` | lines 94.83%, regions 93.68% (floors 92/91) |
| on-chain-client `verify_commands` | 73 passed; three bolero targets ran their 1 s budget |
| on-chain-client `make coverage-on-chain-client` | lines 41.75%, regions 42.60% (floors 41/42; documented accepted shortfall) |
| org-node `verify_commands` | 57 passed; three bolero targets; quint typechecks and five invariants, no violation |
| app `verify_commands` | 78 cargo tests passed; `npm run check` 0 errors (1 pre-existing warning); `npm run test` 30 passed |
| `check-ids.sh` (each unit) | clean |
| `check-trace.sh` (each unit) | exit 0 everywhere; person clean (REQ 10, SDD 3, LLR 15); the only new finding is PR-b7khyw, opened by this change; all others pre-existing |
| `check-units.sh` | clean (5 units, 6 disclaimed) |
| `.github/workflows/rust.yml` | parses (ruby YAML) |
| Coverage, against the class target | class C: statement (lines 100%, regions 100%) and decision (branches 100%, nightly) for person; other units unchanged by this change |
| Working tree | clean |

Problem-ledger delta: opens PR-b7khyw (org-members). Resolves none.

## Red → green

Copied from the task dispatch reports (tasks ran on
`worktree-worktree-person-shared-types` for T1–T7, T9 steps 1–4 and T9c, and on
review-fix task branches `worktree-person-unit-*` thereafter). "Compile" means
the test file failed to build because the item did not exist (E0432 or E0277),
the first red of a new type.

| Item | Test | Watched red |
| --- | --- | --- |
| LLR-64muuw | `crate_is_no_std_and_denies_panics_and_indexing` | its assertions held on the scaffold; proved by mutation instead — removing `#![no_std]` failed it at crate_discipline.rs:9; later, the `indexing_slicing` deny and the root `[workspace.lints.clippy]` table each mutation-proved |
| LLR-eeq89n, REQ-bczz87 | `a_rejected_key_names_its_rule`, `a_rejected_name_names_the_field_and_the_bound`, `rejected_slots_name_the_bound_or_the_order` | failed under mutations that replaced each message with an uninformative one |
| LLR-ayu93n, REQ-r7mytp | `names_are_stored_in_nfc` and the names tests | compile (E0432 `person::Name`) before the type existed; bound-before-NFC mutation failed `a_value_over_the_bound_only_before_normalisation_is_accepted` (names.rs:55) and `a_value_over_the_bound_only_after_normalisation_is_rejected` |
| LLR-7guspr, REQ-q6xkna | `parse_accepts_a_valid_key_and_rejects_what_dalek_rejects`, `a_small_order_point_dalek_accepts_is_rejected`, `parse_rejects_every_small_order_point_dalek_accepts`, `parse_rejects_a_point_with_a_torsion_component` | compile (E0432) before the type; the small-order and non-canonical tests failed with `Ok(DevicePublicKey(..))` before those checks existed; the torsion test failed at device_key.rs:220 before `is_torsion_free`; identity-only small-order mutation failed the eight-torsion test |
| LLR-vs7etb, REQ-3vqs9b | `accepts_a_canonical_x25519_public_key_and_contains_its_bytes`, `rejects_every_small_order_u_coordinate`, `rejects_non_canonical_encodings`, `accepts_p_minus_two` | failed under the interim ed25519 validation (all-zero u accepted, p + 3 accepted, a valid X25519 key refused) before the X25519 rule existed; lowering `FIELD_PRIME` failed `accepts_p_minus_two` |
| LLR-m75m7u, REQ-7gz72r | `accepts_canonical_non_small_order_keys`, `rejects_every_libsodium_small_order_entry` | compile (E0432 `person::x25519`); dropping an order-8 entry with `PersonPublicKey` not delegating failed the full-list test |
| LLR-9p34cv, REQ-4szc22 | the slots tests, `add_keeps_the_set_sorted_wherever_the_key_falls`, `has_device_is_membership_in_the_set` | compile (E0432 `DeviceSlots`); deleting `add_device`'s sort failed slots.rs:84; `has_device` inverted failed its test |
| LLR-sjrh7z, REQ-tq4ms4 | `decoding_accepts_only_strictly_increasing_sets`, `decoding_a_huge_declared_length_stops_at_the_bound` | E0277 (`DeviceSlots: Deserialize`) before the impl; a naive `Vec` decoder failed the huge-length test (`DeserializeUnexpectedEnd` vs `SerdeDeCustom`) |
| LLR-4vsm8d, LLR-6ezhw7, REQ-mu3qgz, REQ-aj6x3n | the device_trie tests, `the_device_root_matches_org_members_today` | compile (E0432 `compute_device_root`); the pinned root `2f45f1f7…d6fd` came from org-members' own code before the move |
| LLR-fbqs2r, LLR-a6krbh, LLR-78t363, LLR-5za6mp, LLR-z99qee (derived) | the Debug/Display/ordering/hash/wire-form tests | each mutation-proved (Display, Debug, `From<…> for String`, hex prefix, Ord reversed, Hash of 31 bytes, Serialize as slice, slots reversed, NodeHash bytes, expecting text) |
| REQ-vxx8k3 | `no_panic.rs` property tests (2048 cases each) | injected panics (non-BMP names, `bytes[0] == 0x42`, an unwrapping Deserialize) each failed a property |
| SDD-k5ee9x, SDD-7r833z, SDD-u9cddc | through their LLRs | — |

## What was wrong, and what was built

Nothing on master was wrong in this unit's terms; `person` did not exist. The
owner's design (grill-requirements, 2026-10-04) moves the validated identity
types out of org-members into a new unit that org-members will depend on, so
that a Person and a Member share one implementation of names, device keys and
the group key. This change builds that unit alone; org-members switches to it in
the following change (branch `worktree-worktree-person-shared-types`). Until
then master holds both copies: the owner accepted that interval explicitly.

Built: 10 requirements, 3 software items, 15 low-level requirements (5 derived),
risk assessments for each, a SOUP inventory, and 73 tests at 100% line, region
and (nightly) branch coverage, with a ratcheted floor of 99/99 enforced by
`make coverage-person` in CI.

Owner rulings taken during this change, each recorded where it applies:
- `PersonPublicKey` is X25519 (the key-agreement root key); `DevicePublicKey`
  is ed25519. The on-chain `orgPubKey` is to become X25519 too (org-node's
  change).
- A `DevicePublicKey` must be canonically encoded, not of small order, and
  torsion-free — prime order only.
- A `PersonPublicKey` rejects small-order and non-canonical u; it accepts twist
  and mixed-order u (RFC 7748 practice).
- Validated types construct through `parse`, with `TryFrom` delegating, as the
  parse-at-the-system-edge ADR states.
- `person` exports the X25519 validity check so org-node's `OrgPublicKey`
  applies it without a copy.
- `person` merges before org-members switches; the temporary parallel copy is
  accepted.

Defect recorded, not fixed here: **PR-b7khyw** (opened) — org-members accepts
small-order, non-canonical and mixed-order ed25519 device keys, contrary to its
SOUP row. Resolved by the switch.

## Review

Seven rounds. The owner ruled a bounded stop after round 6: round 7 runs
normally; if it finds only record or wording items, or test gaps a fix plus a
mutation proof closes without new behaviour, those are fixed and the change
merges without an eighth round. Round 7 met that condition.

### Round 1 (head 0d2970f)

**finding-1**: requirement — REQ-q6xkna (valid ed25519 key) is wider than LLR-7guspr ("exactly when dalek accepts"): `from_bytes` accepts non-canonical encodings, so one point entered DeviceSlots three times under three byte strings.
disposition: owner ruled non-canonical encodings rejected; `parse` re-compresses and compares. Tests `parse_rejects_a_non_canonical_encoding_dalek_accepts` and `no_encoding_of_the_identity_reaches_device_slots`. (Since the torsion-free ruling the check is implied by the others — round 5, finding-41.)

**finding-2**: record — the risk file's claim that the org-members switch narrows nothing a control relies on ignores org-members' byte-comparing key-uniqueness rule, which stops seeing one keypair reused as member key and device key once their encodings differ.
disposition: the risk file records it as a consideration the switch must assess before it merges, and the plan lists it.

**finding-3**: record — org-members' SOUP row claims `VerifyingKey::from_bytes` rejects small-order encodings; the project proved it false, and no problem report existed.
disposition: PR-b7khyw opened in org-members' ledger.

**finding-4**: code — REQ-vxx8k3 (no panic for hostile input) had example-based tests only.
disposition: `person/tests/no_panic.rs`, proptest properties over arbitrary strings, 32-byte arrays and bytes; each mutation-proved with an injected panic.

**finding-5**: code — `the_small_order_list_is_what_it_claims` checks the test's own constant and cannot fail when person/src changes.
disposition: its `verifies:` annotation removed; it is a fixture check, later strengthened to confirm every entry independently by the x-only ladder ([8]u = 0).

**finding-6**: code — LLR-64muuw's "indexes no slice with an input-derived value" was verified by nothing.
disposition: `#![deny(clippy::indexing_slicing)]` in lib.rs; the one hit rewritten as a slice pattern; the crate-discipline test asserts the attribute (mutation-proved).

**finding-7**: code — two annotations did not match their tests (`keys_order_by_their_bytes` on LLR-7guspr; `debug_redacts_personal_data` on LLR-ayu93n).
disposition: re-annotated to the derived LLRs finding-9 created.

**finding-8**: code — Surname's TryFrom and serde decoding untested; no bound test where NFC lengthens a value.
disposition: Surname tests added; a U+0958 test (120 bytes → 240 after NFC) rejected; bound-before-NFC and missing `serde(try_from)` mutations each failed a test.

**finding-9**: requirement — behaviour with no requirement: Debug/Display/`From<…> for String`, key and hash Debug, DeviceSlots Debug, ordering and hashing by bytes, wire forms.
disposition: five derived LLRs (LLR-fbqs2r, LLR-a6krbh, LLR-78t363, LLR-5za6mp, LLR-z99qee), each assessed in the risk file and tested with mutation proofs.

**finding-10**: code — the new validated types did not follow the ADR newtype contract (no `TryFrom`, constructors not named `parse`).
disposition: `TryFrom` impls delegating; then, on the owner's ruling (round 3, finding-31), constructors renamed `parse`.

**finding-11**: record — "group key", an Avoid term, used in the requirements and lib.rs.
disposition: reworded to PersonPublicKey.

**finding-12**: record — the plans were untrue of a person-only branch: "six" requirements, "all four merged with this change", and the parallel copy the ADR forbids not recorded as accepted.
disposition: the plan states the two-part merge and the owner's acceptance of the interval; counts and the units.yaml comment corrected.

**finding-13**: record — the implementation plan's Implements line, draft file paths and commit references were stale for this branch.
disposition: header note on the two branches; Implements line updated.

**finding-14**: record — LLR-ayu93n gave only MAX_NAME_LEN for both fields.
disposition: names MAX_SURNAME_LEN (128) for Surname.

**finding-15**: record — the SOUP serde row omitted REQ-r7mytp.
disposition: added.

### Round 2 (head c0872c0)

**finding-16**: record — curve25519-dalek called "dev-dependency only", but it runs inside `parse` (decompression, re-compression, `is_small_order`).
disposition: safety-relevant runtime SOUP row added.

**finding-17**: code — the exported `is_valid_public_key` was not tested against the full small-order list; only `PersonPublicKey` reached the order-8 values.
disposition: `rejects_every_libsodium_small_order_entry`; dropping an entry with `PersonPublicKey` not delegating failed it.

**finding-18**: code — the fixture check skipped p − 1.
disposition: every canonical entry, p − 1 included, confirmed by [8]u = 0 under curve25519-dalek's ladder, with a basepoint cross-check; mutating an entry or p − 1 failed it.

**finding-19**: code — DevicePublicKey hashing untested; the expecting-text test claimed an LLR that did not state it; accessors had no LLR text.
disposition: hash assertion added (31-byte-hash mutation failed it); LLR-5za6mp names the expected-form message; LLR-9p34cv and LLR-ayu93n name the accessors; `has_device_is_membership_in_the_set` added.

**finding-20**: record — PR-b7khyw named a test that had been renamed.
disposition: updated to `a_small_order_point_dalek_accepts_is_rejected`.

**finding-21**: record — person/docs/CONTEXT.md and the Cargo description claimed the Person definition hash, not on this branch.
disposition: reworded to the identity types; the Person definition comes later.

**finding-22**: requirement — "Person", "Name", "Surname", "device slots", "device root" undefined in any glossary.
disposition: root **Person** entry; Name, Surname, Device slots, Device root in person's glossary.

**finding-23**: record — the Makefile branch-coverage count was stale; the CI comment said decision coverage was measured nowhere.
disposition: refreshed; CI comment says it is not measured in CI.

**finding-24**: record — "org-members is untouched here", though the branch adds a problem report there.
disposition: "its code and requirements are untouched".

**finding-25**: code — Cargo.lock moved from format version 3 to 4.
disposition: restored to version 3; `--locked` builds for person and org-members confirmed.

### Round 3 (head 65a9a31)

**finding-26**: code — DevicePublicKey small-order tests covered only the identity.
disposition: `parse_rejects_every_small_order_point_dalek_accepts` over all eight torsion points from `EIGHT_TORSION`; an identity-only mutation failed it.

**finding-27**: code — no acceptance case just below the X25519 bound; Surname at its bound untested.
disposition: `accepts_p_minus_two` in both test files (a lowered `FIELD_PRIME` failed only these); Surname at exactly MAX_SURNAME_LEN tested and mutation-proved.

**finding-28**: requirement — LLR-eeq89n claimed every rejection is an IdentityError variant; decoding rejects through serde's error.
disposition: narrowed to in-process constructors; decoding messages that name the rule are tested in `decode_errors.rs` through `serde::de::value` (postcard drops messages).

**finding-29**: record — the CI comment said branch coverage was measured "at the person merge gate".
disposition: reworded to where and when it was measured.

**finding-30**: record — T9c had no Done note; T9's note used pre-rename names.
disposition: T9c Done note added; T9's note marked historical.

**finding-31**: requirement — the ADR says validated types have `parse`; the new types used `from_bytes` and `new`.
disposition: owner chose rename to `parse` (2026-10-05); `new(VerifyingKey)` replaced by `TryFrom<VerifyingKey>` delegating to `parse`; the other branch renames its call sites when it merges master.

**finding-32**: record — items with no invalid-input class were not recorded as such.
disposition: "Items with no invalid input" section in the architecture ledger.

### Round 4 (head cdfd383)

**finding-33**: record — the identity point has four encodings dalek accepts, not three.
disposition: count fixed; the fourth case added to the test.

**finding-34**: record — Makefile coverage counts stale (215/324 vs 214/326).
disposition: refreshed.

**finding-35**: code — `every_rejection_is_a_distinct_named_variant` cannot detect a constructor returning the wrong variant.
disposition: annotation removed; labelled a fixture check of the enum.

**finding-36**: requirement — LLR-4vsm8d's "not object-safe" clause is not testable behaviour.
disposition: moved to prose after the item, with its reason.

**finding-37**: record — LLR-64muuw missing from "Items with no invalid input".
disposition: added, with the no-panic properties and lint configuration as its robustness evidence.

**finding-38**: record — person's clippy verify command ran nowhere in CI.
disposition: person added to the test and clippy jobs.

**finding-39**: requirement — mixed-order device keys (prime point + torsion) accepted, though device keys are to sign admin approvals.
disposition: owner ruled torsion-free required (2026-10-05); `parse` adds `is_torsion_free`; `parse_rejects_a_point_with_a_torsion_component` failed before the check and under its removal.

**finding-40**: record — "device-set validation" (Avoid term) and "can never coincide" overstated.
disposition: "device-slots validation"; "up to the hash function's collision resistance".

**finding-41**: record — the plan's self-review list omitted REQ-7gz72r and LLR-m75m7u; the T9c snippet was superseded code.
disposition: list updated; snippet marked historical.

**finding-42**: record — ledger files dated 2026-10-04 while the merge lands on 2026-10-05.
disposition: no change — per merge-change step 3 the date in the name is the day the draft name was retired, not the merge date.

### Round 5 (head 562eb01)

**finding-43**: code — `parse_rejects_a_non_canonical_encoding_dalek_accepts` cannot detect removal of the canonical check: since the torsion-free ruling every non-canonical encoding dalek accepts is small-order or torsioned (checked over y = 0..18 with exact arithmetic).
disposition: check kept as defence in depth against a change in dalek's decoding; LLR-7guspr, the test comment and the SOUP rows state it is implied by the other two checks and verified by inspection; the test asserts the torsion component of y = p + 3.

**finding-44**: requirement — twist u-coordinates accepted by PersonPublicKey without a record.
disposition: owner accepted (RFC 7748 practice, twist security); stated after REQ-3vqs9b, in LLR-vs7etb and LLR-m75m7u and the risk file; `twist_u_coordinates_are_accepted` computes twist membership by Legendre symbol (mutation-proved).

**finding-45**: record — the hand-off to the org-members switch listed two narrowings, not three (mixed-order device keys).
disposition: risk file and PR-b7khyw list all three.

**finding-46**: record — the CI comment said "both" coverage targets and cited an old branch measurement.
disposition: "all three"; cites the Makefile's 28/28.

**finding-47**: record — LLR-64muuw and person's rust-version enforced by no build.
disposition: person wasm32 checks in the no-std job and person in the msrv job.

**finding-48**: record — ledger file dates.
disposition: no change, as finding-42.

### Round 6 (head f571a6c)

**finding-49**: code — `add_device`'s sort untested: the only successful add appended a key that already sorted last.
disposition: `add_keeps_the_set_sorted_wherever_the_key_falls`; deleting the sort failed it at slots.rs:84.

**finding-50**: code — the LLR-64muuw test did not check the workspace's deny list.
disposition: it parses the root `[workspace.lints.clippy]` table and asserts unwrap_used, expect_used and panic = "deny"; relaxing each failed it.

**finding-51**: record — mixed-order X25519 keys: one secret maps to up to eight accepted byte-distinct PersonPublicKeys.
disposition: recorded in the risk file and the plan as a second consideration the switch must assess (org-members' byte-comparing key rule).

**finding-52**: requirement — REQ-q6xkna equated torsion-free with prime order (the identity is torsion-free).
disposition: "a point of prime order (torsion-free and not the identity)".

**finding-53**: requirement — REQ-4szc22, REQ-vxx8k3 and REQ-mu3qgz each bundled two behaviours.
disposition: split in place (unmerged items): REQ-tq4ms4, REQ-bczz87 and REQ-aj6x3n minted, assessed, and traced; annotations moved.

**finding-54**: record — "member key" (Avoid term) in the risk file and PR-b7khyw.
disposition: replaced by "Member-as-a-group key".

**finding-55**: record — the no-std CI comment claimed wasm32 proves no_std (wasm32 has std).
disposition: reworded: `#![no_std]`, checked by the crate-discipline test, is the proof.

**finding-56**: record — T10 told the reader to drop the curve25519-dalek row.
disposition: T10 marked historical and superseded.

**finding-57**: record — person's config comment said "one test file per software item".
disposition: corrected to how the tests are split.

### Round 7 (head 20407c8) — closed under the owner's bounded-stop rule

**finding-58**: requirement — PersonPublicKey's acceptance of mixed-order u-coordinates (owner decision) was recorded only in the risk file; LLR-vs7etb named only twist acceptance and no test fed a mixed-order u.
disposition: LLR-vs7etb and LLR-m75m7u state it; `accepts_mixed_order_u_coordinates` builds basepoint + each non-identity torsion point and asserts acceptance by both functions; rejecting one such u failed only this test.

**finding-59**: code — three no-panic property tests claimed LLR-eeq89n (and the decoding test three key LLRs) without asserting those behaviours.
disposition: annotations narrowed to what each asserts; LLR-eeq89n remains verified by decode_errors.rs and slots_decode.rs; check-trace clean.

**finding-60**: record — PR-b7khyw's symptom named only small-order and non-canonical keys, and "all three cases" read as applying to member keys too.
disposition: symptom names mixed-order device keys; "all three" limited to device keys.

**finding-61**: code — LLR-sjrh7z's "rejects before storing a key beyond the bound" cannot be distinguished by a test from push-then-check.
disposition: recorded in LLR-sjrh7z as verified by inspection (the visitor checks the length before each push).

**finding-62**: record — soup.md said "As built 2026-10-04" and gave no versions for the blake3 and postcard dev-dependencies.
disposition: "As built at the merge (2026-10-05, after the torsion-free ruling)"; blake3 1.8.7 and postcard 1.1.3 from Cargo.lock.

**finding-63**: record — the units.yaml comment described person as an individual's own definition.
disposition: describes the shared identity types and the X25519 check it holds now; the Person definition comes later.

## Gaps

- org-node and app have no coverage command (pre-existing, recorded in their
  configs); on-chain-client's floors sit on its documented accepted shortfall.
- The canonical-encoding check in `DevicePublicKey::parse` and the
  check-before-store clause of LLR-sjrh7z are verified by inspection only: no
  input can tell them apart from the other checks.
- Decision (branch) coverage is measured by hand on nightly (28/28), not in CI.
- The MSRV (1.85) check for person runs in CI only; the toolchain is not
  installed locally, so no local run backs it.
- `person` is not yet used by any unit; the switch must assess the two key-reuse
  considerations recorded in the risk file (reused keypair across roles;
  mixed-order X25519 encodings) before it merges.
- Tests were run with `QUINT_HOME` set to a scratch directory seeded from
  `~/.quint` and `CARGO_HOME=/tmp/cargo_home_fuzz` for org-node, app and
  on-chain-client (read-only default homes in this environment).
