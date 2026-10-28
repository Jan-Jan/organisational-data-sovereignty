# Verification — worktree-generic-smt, the sparse Merkle tree unit (2026-10-28)

One record per change, written at `merge-change` step 5 and checked at step 6c
by `.guardrails/scripts/check-review.sh`. The squash commit references it on
its `Verified:` line and carries the gate's figures in its own trailers; this
file is the rest of the evidence that travels with the change.

branch: worktree-generic-smt
reviewer: independent subagent (fresh context, no chat history), merge-change step 4, 2026-10-28
verdict: no code or requirement finding above low — last review round; four low findings fixed
reproduced: no — a new capability, not a defect: the gap (device revocation not provable under a device root or a Person hash) was established by design review, not reproduced as a failing test

Units this change touches: sparse-merkle-tree (new), person and org-members
(product code, tests, ledgers); small edits in org-node's tests
(`org-node/tests/encoding_golden.rs`), `app/src-tauri/Cargo.lock` (its own
workspace's lock follows the new path dependency) and on-chain-client's
configuration (`on-chain-client/.guardrails/config.yaml`). The impact set,
as `sh .guardrails/scripts/check-units.sh --impact refs/heads/master..HEAD`
printed it:

    org-members	touched
    on-chain-client	touched
    org-node	touched
    app	touched
    person	touched
    org-io	touched
    sparse-merkle-tree	touched

Change: a new `sparse-merkle-tree` unit (one immutable sparse Merkle tree of
a compile-time depth, generic over its leaf and hasher, with absence and
inclusion proofs) on which person's device root and org-members' Membership
record are rebuilt, with two-step absence proofs under a Membership root and
under a Person hash.
Plan: `docs/plans/2026-10-24-sparse-merkle-tree.md`.

Implements (the plan's **Implements:** value): sparse-merkle-tree
REQ-vg9wq4, REQ-6cej5z, REQ-uy4euz, REQ-ymgg6b, REQ-s3unp5, REQ-dyjq33,
REQ-3szczq, REQ-fmx9e7; SDD-x43d2z, SDD-rh9u6z, SDD-ur6uvu; LLR-jd4phg,
LLR-znpbf5, LLR-t3e8pp, LLR-9z6dc9, LLR-ggu9bh, LLR-mm45ry, LLR-yuyn8f,
LLR-kep2ju, LLR-8uem8c, LLR-ycf27m, LLR-3y5pq6, LLR-3zffc2, LLR-u2r4vj,
LLR-byz55h, LLR-c4eunm, LLR-9qncuc, LLR-639mz9, LLR-m5c3vw, LLR-tz3nfz,
LLR-5tww69, LLR-88rm99, LLR-77zy97, LLR-53ezgd, LLR-c7bdee, LLR-smf4zq.
person REQ-5kvqcf, REQ-uh2bzg, REQ-9sh8qb, REQ-4e23xr; SDD-bx9mm6,
SDD-7r833z and SDD-k5ee9x (amended); LLR-9mjfjn, LLR-8y8tqr, LLR-exwx4n,
LLR-vraz7n, LLR-xsx6gg, LLR-pjf9qn, LLR-qwp8d7, LLR-985v9t, LLR-cve3d6,
LLR-z8h2w8, LLR-3dvfjb. org-members REQ-sb2frw, REQ-65xrgh, REQ-2sz7j8;
SDD-d9svdj and SDD-57vaj4 (amended); LLR-t465mb, LLR-4rdqyn, LLR-uudst2,
LLR-utk6uy, LLR-5jh6t3, LLR-w5yahk, LLR-hbb4je, LLR-v6smww, LLR-bh53rr,
LLR-9g2kjg, LLR-54gp2f, LLR-25nmwc, LLR-v3cysh. Through the REQs it
realises RC-5zyczk, RC-gy5f5r (sparse-merkle-tree), RC-ja7gx3, RC-5d9qxe
(person), RC-5nka8z, RC-j2znx8 (org-members). LLR-cyx9a3 (person) was
minted during the CODE review and is implemented too.

IDs this change supersedes, as the ledgers record them:

- REQ-535jcd → REQ-sb2frw and REQ-tk2qqj → REQ-65xrgh
  (`org-members/docs/requirements/2026-10-23-two-level-absence-proofs.md`);
- HAZ-gwbn5n → HAZ-r5tc4m
  (`org-members/docs/risk/2026-10-23-two-level-absence-proofs.md`);
- org-members (`org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md`):
  LLR-72p8bz → LLR-4rdqyn, LLR-kdhd2v → LLR-uudst2, LLR-pys2ek → LLR-utk6uy
  (hashing and the device root); LLR-25tpdp → LLR-v6smww,
  LLR-4xz255 → LLR-bh53rr, LLR-4rju5r → LLR-9g2kjg, LLR-dgzy7e → LLR-54gp2f,
  LLR-p2p8qy → LLR-25nmwc, LLR-2dcnbp → LLR-v3cysh (the absence proof);
  LLR-28ekrv → LLR-hbb4je (the mapping of person's errors). Retired:
  LLR-utp6x4, LLR-7jkcba. SDD-d9svdj and SDD-57vaj4 amended in place;
- person (`person/docs/architecture/2026-10-24-device-root-tree.md`):
  LLR-4vsm8d → LLR-9mjfjn, LLR-6ezhw7 → LLR-8y8tqr, LLR-9p34cv → LLR-xsx6gg,
  LLR-edn55h → LLR-qwp8d7, LLR-5za6mp → LLR-cyx9a3. Retired: LLR-z99qee.
  SDD-7r833z and SDD-k5ee9x amended in place.

It resolves no problem item, accepts none and opens none.

## The gate

The gate's figures are not in this record. The squash commit carries them as
trailers (`merge-change` step 7): `Gate-tree:` names the tree the step 6a gate
measured, a `Gate:` line gives each command's totals, and `Base:` names the
base and the commit step 1 merged. `finish-merge.sh` refuses the cleanup
unless `Gate-tree:` is the commit's own tree.

## Red → green

One row per ID this change implements, from the `red -> green:` lines of the
plan's task reports (T2–T11) and of the guideline-review fixes. "Compile red"
means the test's file failed to build for the missing API before the
implementation existed, which is red for the right reason but not a failed
assertion; rows say so where that is the only red, where red was observed
only against a mutant of existing behaviour, and where a test was not
watched red at all. REQ and SDD rows name the tests of the LLRs that carry
them.

| Item | Test | Watched red |
| --- | --- | --- |
| `REQ-vg9wq4` | through LLR-ggu9bh, LLR-mm45ry, LLR-kep2ju, LLR-9qncuc: `the_root_is_the_classic_trees_at_depth_256`, `depths_one_and_256_build` | compile red only (T3: E0432 unresolved import `sparse_merkle_tree::SparseMerkleTree`) |
| `REQ-6cej5z` | `insert_refuses_a_collision_and_replaces_the_same_identity` | compile red only (T3: E0432) |
| `REQ-uy4euz` | `prove_absent_ends_by_what_the_position_holds` | compile red only (T5: unresolved `AbsenceProof`/`ProofEnding`, no method `prove_absent`) |
| `REQ-ymgg6b` | `verify_absent_checks_the_root_first_then_the_ending` | compile red only (T5: unresolved `AbsenceProof`) |
| `REQ-s3unp5` | `an_absence_proof_holds_a_map_its_siblings_and_an_ending` | compile red only (T5: unresolved `AbsenceProof`/`ProofEnding`) |
| `REQ-dyjq33` | `decoding_and_verifying_any_bytes_does_not_panic`, `from_parts_refuses_a_map_or_siblings_that_do_not_fit_the_depth`, `any_operation_sequence_does_not_panic` | compile red only (T6: `Serialize`/`Deserialize` not implemented; T5: unresolved proof types); `any_operation_sequence_does_not_panic` not red (passed on first run over T3–T5 code) |
| `REQ-3szczq` | `prove_included_returns_the_leaf_and_a_proof_that_verifies` | compile red only (T5: no method `prove_included`) |
| `REQ-fmx9e7` | `an_inclusion_proof_refuses_any_other_leaf_or_root` | compile red only (T5) |
| `SDD-x43d2z` | through LLR-jd4phg, LLR-znpbf5, LLR-t3e8pp, LLR-9z6dc9, LLR-byz55h, LLR-c4eunm (rows below) | as those rows |
| `SDD-rh9u6z` | through LLR-ggu9bh … LLR-639mz9, LLR-u2r4vj (rows below) | as those rows |
| `SDD-ur6uvu` | through LLR-m5c3vw … LLR-smf4zq (rows below) | as those rows |
| `LLR-jd4phg` | `a_node_hash_wraps_any_32_bytes`, `a_node_hash_encodes_as_its_32_bytes` | red before the crate existed (T2: package ID specification `sparse-merkle-tree` did not match any packages) |
| `LLR-znpbf5` | `a_depth_one_root_is_built_from_the_three_hasher_functions` | compile red only (T3: E0432) |
| `LLR-t3e8pp` | `only_the_identity_hash_and_content_hash_enter_the_tree` | compile red only (T3: E0432) |
| `LLR-9z6dc9` | `a_depth_one_root_is_built_from_the_three_hasher_functions`, `only_the_identity_hash_and_content_hash_enter_the_tree` | compile red only (T3: E0432) |
| `LLR-ggu9bh` | `depths_one_and_256_build`; the `compile_fail` doctests in `sparse-merkle-tree/src/tree.rs` | `depths_one_and_256_build` compile red only (T3: E0432); the depth-0 doctest fails only on E0080 "a sparse Merkle tree's depth is 1 to 256"; the depth-257 doctest was added at the merge review (Review, Round 1, finding-4) and not watched red |
| `LLR-mm45ry` | `positions_are_the_first_depth_bits_most_significant_first` | compile red only (T3: E0432) |
| `LLR-yuyn8f` | `the_empty_subtree_hashes_and_their_bound` | compile red only (T3: E0432) |
| `LLR-kep2ju` | `the_root_is_the_classic_trees_at_depth_8`, `the_root_is_the_classic_trees_at_depth_256` | compile red only (T3: E0432) |
| `LLR-8uem8c` | `insert_and_remove_leave_the_receiver_unchanged` | compile red only (T3: E0432) |
| `LLR-ycf27m` | `removal_gives_the_classic_tree_of_what_remains`, `remove_refuses_an_identity_not_held` | compile red only (T3: E0432) |
| `LLR-3y5pq6` | `hashes_are_write_once_and_the_root_waits_for_them` | compile red only (T3: E0432) |
| `LLR-3zffc2` | `the_root_depends_on_the_leaf_set_alone` | compile red only (T3: E0432) |
| `LLR-u2r4vj` | `changes_between_reports_removed_added_and_replaced_in_position_order`, `equal_trees_differ_in_nothing`, `changes_between_walks_uncalculated_trees_and_lowers_compact_leaves`, `changes_between_matches_the_set_difference` | compile red only (T4: E0599 no associated function `diff`; the tests were renamed from `diff_…` when the function became `changes_between`) |
| `LLR-byz55h` | `the_crate_is_no_std_and_denies_indexing`, `the_source_names_no_std_item_and_no_io` | red before the crate existed (T2) |
| `LLR-c4eunm` | `the_only_dependencies_are_spin_and_optional_serde` | red before the crate existed (T2) |
| `LLR-9qncuc` | `from_leaves_builds_a_calculated_tree_or_refuses_a_collision` | compile red only (T3: E0432) |
| `LLR-639mz9` | `get_contains_and_leaves_read_without_changing_the_tree` | compile red only (T3: E0432) |
| `LLR-m5c3vw` | `an_absence_proof_holds_a_map_its_siblings_and_an_ending` | compile red only (T5) |
| `LLR-tz3nfz` | `prove_included_returns_the_leaf_and_a_proof_that_verifies` | compile red only (T5: no method `prove_included`) |
| `LLR-5tww69` | `prove_absent_ends_by_what_the_position_holds`, `prove_absent_refuses_a_held_leaf_and_an_uncalculated_tree`, `proofs_verify_at_depth_256_at_the_edges` | compile red only (T5: no method `prove_absent`) |
| `LLR-88rm99` | `prove_included_refuses_an_absent_leaf_and_an_uncalculated_tree` | compile red only (T5) |
| `LLR-77zy97` | `proofs_round_trip_and_still_verify`, `a_proof_parses_at_its_own_depth_only`; `a_depth_four_map_is_refused_for_each_malformed_shape`, `decoding_refuses_a_value_that_is_not_the_tuple`, `encoding_into_a_short_buffer_fails` | the first two compile red only (T6: `Serialize`/`Deserialize` not implemented for `AbsenceProof<8>`/`InclusionProof<8>`, then depth 4); the last three (T11) red only against mutants of existing behaviour: the map-length check disabled (proofs.rs:183), decoding through `deserialize_any` with `visit_str` (proof_decoding.rs:89), the `serialize_element` `?`s swallowed (proof_decoding.rs:107) |
| `LLR-53ezgd` | `verify_absent_checks_the_root_first_then_the_ending` | compile red only (T5) |
| `LLR-c7bdee` | `an_inclusion_proof_refuses_any_other_leaf_or_root` | compile red only (T5) |
| `LLR-smf4zq` | `decoding_and_verifying_any_bytes_does_not_panic`, `decoding_refuses_malformed_and_oversized_parts`, `a_huge_declared_length_reserves_nothing`, `every_refusal_is_its_own_variant_and_message`, `any_operation_sequence_does_not_panic` | the first three compile red only (T6); `every_refusal_is_its_own_variant_and_message` compile red of its own file only — the `TreeError` it checks existed from T2; `any_operation_sequence_does_not_panic` not red |
| `REQ-5kvqcf` | `a_proof_for_a_key_not_held_verifies_against_the_person_hash`, `no_proof_is_made_for_a_key_the_person_holds` | compile red only (T8: unresolved `person::prove_device_absent`, `person::PersonAbsenceProof`) |
| `REQ-uh2bzg` | `verify_checks_the_person_hash_first`, `verify_then_checks_the_device_root_and_the_ending` | compile red only (T8) |
| `REQ-9sh8qb` | `a_position_is_the_keyed_hash_under_the_persona_domain`, `add_and_remove_return_new_sets_and_refuse_by_rule` | compile red only (T7: E0432 unresolved `person::device_position`, `device_tree`, `DeviceRootHasher`, `DEVICE_POSITION_KEY`, `DEVICE_ROOT_DEPTH`) |
| `REQ-4e23xr` | `decoding_refuses_two_keys_of_one_position`, `parse_refuses_by_rule_in_order` | compile red only (T7: no variant `IdentityError::DevicePositionTaken`) |
| `SDD-bx9mm6` | through LLR-985v9t, LLR-cve3d6, LLR-z8h2w8, LLR-3dvfjb (rows below) | as those rows |
| `SDD-7r833z` | through LLR-9mjfjn, LLR-8y8tqr, LLR-exwx4n (rows below) | as those rows |
| `SDD-k5ee9x` | through LLR-qwp8d7 (row below) | as that row |
| `LLR-9mjfjn` | `the_root_is_the_depth_four_tree_of_the_keys_at_their_positions`, `the_domain_keys_and_sentinel_are_the_documented_bytes` | compile red only (T7: E0432) |
| `LLR-8y8tqr` | `equal_sets_give_equal_roots_whatever_the_order`, `the_person_device_root_is_pinned`, `the_tree_holds_one_device_leaf_per_key` | compile red only (T7); the pin red on "RE-PIN", then pasted; `the_tree_holds_one_device_leaf_per_key` (T11) red only against a mutant of `device_tree` keeping only the first key (device_root.rs:165) |
| `LLR-exwx4n` | `a_position_is_the_keyed_hash_under_the_persona_domain` | compile red only (T7: E0432 `person::device_position`) |
| `LLR-vraz7n` | `add_and_remove_return_new_sets_and_refuse_by_rule` | compile red only (T7: E0599 no method `position_taken`) |
| `LLR-xsx6gg` | `parse_refuses_by_rule_in_order`, `parse_sorts_and_accepts_zero_to_the_bound`, `try_from_goes_through_parse` (person/tests/slots.rs) | compile red only (T7); the restored accepting case of `try_from_goes_through_parse` (TEST round 5) red only against a mutant refusing one-element sets ("a valid set is accepted: DeviceSlotsFull" at slots.rs:121) |
| `LLR-pjf9qn` | `decoding_refuses_two_keys_of_one_position` | compile red only (T7) |
| `LLR-qwp8d7` | `the_v1_encoding_is_the_documented_layout`, `the_v1_hash_is_pinned`, `no_domain_key_or_the_sentinel_is_org_members` | compile red only (T7); the pin red on "RE-PIN", then pasted; `no_domain_key_or_the_sentinel_is_org_members` (TEST round 4) red only against `PERSON_DEVICE_NODE_KEY` set to org-members' key |
| `LLR-985v9t` | `a_proof_holds_the_fields_and_no_device_public_key`, `a_proof_survives_the_wire_and_malformed_ones_are_refused` | compile red only (T8) |
| `LLR-cve3d6` | `no_proof_is_made_for_a_key_the_person_holds`, `every_proof_made_verifies` | compile red only (T8) |
| `LLR-z8h2w8` | `verify_then_checks_the_device_root_and_the_ending` | compile red only (T8) |
| `LLR-3dvfjb` | `the_proof_refusals_are_distinct_named_variants`, `no_public_signature_names_the_tree_or_its_error` | `the_proof_refusals_are_distinct_named_variants` compile red of its own file only (T8; the variants were added in T7), and, extended in TEST round 1, red against a mutant giving `DeviceNotFound` `DeviceStillHeld`'s message (person_absence_proofs.rs:154); `no_public_signature_names_the_tree_or_its_error` red with `device_tree`/`DeviceTree` made pub again (ARCHITECTURE round 1), then, extended in rounds 2 and 3, red against mutants `IdentityError::Tree(TreeError)`, `impl From<TreeError> for IdentityError`, a pub `SparseMerkleTree` field and a scanner skipping enum variants |
| `REQ-sb2frw` | `prove_absent_refuses_a_device_the_member_holds`, `prove_absent_ignores_a_key_another_member_holds`, `every_honest_absence_proof_verifies` | absence_proofs.rs compile red only (T9: no `is_vacant`/`member`/`inclusion`/`device_absence`/`vacancy` on `AbsenceProof`, `Blake3Hasher` not `TreeHasher`); the fuzz property's Held case not run before the implementation |
| `REQ-65xrgh` | `honest_proofs_verify`, `a_proof_is_refused_against_another_root`, `every_honest_absence_proof_verifies` | compile red only (T9); the fuzz property not run before the implementation |
| `REQ-2sz7j8` | `a_device_at_a_taken_position_is_refused`, `scenario_device_position_taken`, Quint `scenarioDevicePositionTaken` | red only as a library compile failure (T7: E0432 `person::DeviceTrieHasher`, org-members src/hasher.rs:1), not on their own assertions; the Quint scenario not red (written with the model change in one edit) |
| `SDD-d9svdj` | through LLR-t465mb, LLR-4rdqyn, LLR-w5yahk (rows below) | as those rows |
| `SDD-57vaj4` | through LLR-v6smww … LLR-v3cysh (rows below) | as those rows |
| `LLR-t465mb` | `a_member_leaf_hashes_as_documented`, `member_set_root_is_pinned` | compile red only (T9: no `member_identity`/`member_content`); the pin red on "RE-PIN" in T7 and again in T9, then pasted |
| `LLR-4rdqyn` | `hasher_domains_are_separated`, `no_device_set_shares_its_root_with_the_person_device_root` | `hasher_domains_are_separated` compile red only (T9), then, extended in TEST round 4, red against mutants changing the device-node key and the sentinel; `no_device_set_shares_its_root_with_the_person_device_root` red against a mutant giving org-members person's device domains and sentinel |
| `LLR-uudst2` | `every_device_reaches_the_root` | red only as a library compile failure (T7: E0432 `person::DeviceTrieHasher`) |
| `LLR-utk6uy` | `every_device_reaches_the_root`; inherited `add_p2p_device_rejects_when_full`, `member_leaf_too_many_devices`, `deserialize_rejects_too_many_devices` | `every_device_reaches_the_root` red only as a library compile failure (T7); the inherited tests were re-pointed, not watched red |
| `LLR-5jh6t3` | `a_device_at_a_taken_position_is_refused`, `scenario_device_position_taken`, Quint `addDevicePositionTakenTest`, `addDevicePositionBeforeDuplicateKeyTest`, `scenarioDevicePositionTaken` | the Rust tests red only as a library compile failure (T7); the Quint tests not red (written with the model change in one edit) |
| `LLR-w5yahk` | `every_tree_error_maps_to_an_org_members_error`, `no_public_signature_returns_or_holds_the_tree_error` | compile red only (T9: no `From<TreeError> for OrgMembersError`); `no_public_signature_returns_or_holds_the_tree_error` (ARCHITECTURE round 2) red against mutants: a pub fn returning `TreeError` on `OrgMembersError`, a pub enum variant `Tree(TreeError)`, a scanner skipping enum variants |
| `LLR-hbb4je` | `person_only_variants_map_to_invariant_violated`, `shared_variants_keep_their_names_and_fields` | inherited from LLR-28ekrv; the T7 additions red only as a library compile failure (T7) |
| `LLR-v6smww` | `a_vacant_proof_carries_no_member_data`, `a_held_proof_carries_the_member_without_device_keys` | compile red only (T9) |
| `LLR-bh53rr` | `prove_absent_refuses_an_uncalculated_trie`, `prove_absent_refuses_a_device_the_member_holds` | compile red only (T9) |
| `LLR-9g2kjg` | `the_wire_refuses_malformed_proofs`, `absence_proof_decoding_and_verify_never_panic` | compile red only (T9); the fuzz property with the new `arb_wire_proof` not run before the implementation |
| `LLR-54gp2f` | `a_proof_is_refused_for_another_member_or_device` | compile red only (T9) |
| `LLR-25nmwc` | `a_proof_ending_in_the_asked_device_is_refused_as_held`, `a_vacant_proof_ending_in_the_member_is_refused` | compile red only (T9) |
| `LLR-v3cysh` | `a_proof_is_refused_against_another_root` | compile red only (T9) |
| `LLR-cyx9a3` | `a_key_encodes_as_its_32_bytes` (device_key.rs, person_key.rs), `slots_encode_as_the_sequence_of_their_keys_in_sorted_order`, `decoding_a_non_sequence_names_the_expected_form` | not red: minted in the CODE round-1 fixes to reword LLR-5za6mp's `expecting` message; its tests are inherited from LLR-5za6mp and were re-pointed |

Other tests watched red on this branch, verifying items in force before it:
`a_member_is_addressed_by_its_id_bits_most_significant_first` (T9, compile
red, no `AbsenceProof::vacancy`), `the_extreme_member_ids_are_handled` (T9,
red only as its binary's compile failure), `revocation_wire_message_is_pinned`
in org-node (T10, red decoding the old `GOLDEN_REVOCATION` with
`MalformedMessage`, re-pinned to the derived value) and
`public_signatures_are_read_whole` (ARCHITECTURE round 2, red against the
old helper; its cases were later folded into the annotated scan tests).

inherited: person_only_variants_map_to_invariant_violated — from LLR-28ekrv
inherited: shared_variants_keep_their_names_and_fields — from LLR-28ekrv
inherited: device_slot_order_does_not_change_the_root — from LLR-kdhd2v; deleted at the merge review (Review, Round 1, finding-3)
inherited: every_honest_absence_proof_verifies — from LLR-utp6x4, LLR-dgzy7e, LLR-4xz255
inherited: member_leaf_too_many_devices — from LLR-pys2ek
inherited: deserialize_rejects_too_many_devices — from LLR-pys2ek
inherited: add_p2p_device_rejects_when_full — from LLR-pys2ek
inherited: add_keeps_the_set_sorted_wherever_the_key_falls — from LLR-9p34cv
inherited: has_device_is_membership_in_the_set — from LLR-9p34cv
inherited: try_from_goes_through_parse — from LLR-9p34cv
inherited: debug_is_the_count_and_names_no_key — from LLR-9p34cv
inherited: absence_proof_decoding_and_verify_never_panic — from LLR-2dcnbp, LLR-4rju5r
inherited: device_key.rs::a_key_encodes_as_its_32_bytes — from LLR-5za6mp
inherited: person_key.rs::a_key_encodes_as_its_32_bytes — from LLR-5za6mp
inherited: names.rs::names_encode_as_the_string_of_their_nfc_value — from LLR-5za6mp
inherited: slots_decode.rs::slots_encode_as_the_sequence_of_their_keys_in_sorted_order — from LLR-5za6mp
inherited: slots_decode.rs::decoding_a_non_sequence_names_the_expected_form — from LLR-5za6mp

## What was wrong, and what was built

The gap: a revoked Device could not be shown revoked by proof. person's
device root was a fixed four-slot hash with no absence proof, so nothing
proved a DevicePublicKey absent under a device root or under a Person hash,
and org-members' Membership record, a hand-written sparse trie of its own,
proved a MemberId vacant but could not carry that proof down to a Member's
Devices. Two copies of a Merkle tree, neither able to prove a Device gone,
was the measured state (design review, 2026-10-23/24,
`docs/adr/2026-10-23-one-sparse-merkle-tree.md`).

What was built, as the change stands at merge:

- `sparse-merkle-tree`, a `no_std` class C unit depending only on `spin`
  and optional `serde`: one immutable sparse Merkle tree of a compile-time
  depth in 1..=256 (a depth outside it fails to compile), generic over its
  leaf and hasher, storing no empty nodes, holding single leaves as compact
  leaves at any level while its root stays the classic fixed-depth tree's,
  with write-once hashes, a tree-changes walk (`changes_between`) over
  calculated and uncalculated trees, and absence and inclusion proofs typed
  by their depth whose serde form carries the depth and refuses any other.
  Leaves are unsalted (`docs/adr/2026-10-23-unsalted-tree-leaves.md`).
- person's device root is the root of a depth-4 device root tree, each
  DevicePublicKey at its device position (the first 4 bits of a keyed hash
  under the persona position domain); the bound on DevicePublicKeys is 8
  and two keys of one position are refused (`DevicePositionTaken`); the V1
  Person hash keeps its name and is defined over the new device root
  (owner ruling 2026-10-23); and a Person absence proof shows a
  DevicePublicKey absent under a Person hash, carrying no DevicePublicKey.
- org-members' Membership record is a depth-256 tree of that unit, its
  leaf a `MemberLeaf` with its device root under eight separated domains;
  its absence proof is two-step — Vacant (the MemberId is not held) or Held
  (the Member is held and the DevicePublicKey is absent under its device
  root) — and every error of the tree crate maps exhaustively onto an
  `OrgMembersError` variant named for the check it reports.

Owner rulings that shaped the result: person's API was narrowed so no
person signature names the tree or its error
(`prove_device_absent_in::<H>` replaces the exported device tree, with an
architecture test; 2026-10-27); the 32 bytes a position is read from are
"position bytes" (`Leaf::position_bytes`, 2026-10-27); Boundaries 2 (a
mapped error keeps no cause, and the tree's proof types are shared with
both consumers by design) and the spread of serde derives plus one wire
module per crate were accepted as designed (2026-10-28); "Absence proof" is
the general term, with Tree, Device and Person absence proofs and Vacant and
Held as the two Device absence proofs, and the field the Held proof keeps
is `device_root_absence` (2026-10-28).

Golden values re-pinned on purpose, never to make a red test pass, each
after structural tests recomputed it from `blake3::keyed_hash` alone:

| Value | Old | New |
| --- | --- | --- |
| Person device root, no devices | `db36a2843de361cd8fa691c95f305e206fbcdc666e0f1d1a5e494610df4421a5` | `a01d79a58bbd2f5c628513a1c31a338d9f84a623a608dd2979be451f433fdac0` |
| Person device root, devices 1, 2 | `6b8a4eea74c7daf62a0e6b87112f24ad1037bbfdc5e8607eaa2e217bc8e9d4a5` | `1adf410eb24c2d605b0fff5c81928567158a9100611ce790c565e744b64f4f15` |
| V1 Person hash, Alice Example, key 9, devices 1, 2 | `d6177f1cbed18504b63666dd45b5f54834667606fa73de40c2dbaf28a704e292` | `0d8019c330b4279c02cf747b911a9cb6f1357dbb164bbd9183265e90ca951c8a` |
| V1 Person hash, Alice Example, no key, no device | `225bc8ac0117ae0b3c4aaaec7654b81b3aa45a8f27a58eab7f365dd6fc8183f0` | `943411327947df5cd3f96b581d5c466cb0c8202ff00d72516fda45e4bbdd8bc3` |
| member-set root (GOLDEN_ROOT), first re-pin (T7, new device roots) | `4740435a055ef3825a1a96a9a72361c3bc22bd2c47a992cda21161048d17efd4` | `b0c01649b9f0c8169c5d47b9c0177d37d44f56e04ab5bfab4192cfbee70909d7` |
| member-set root (GOLDEN_ROOT), second re-pin (T9, new member leaf hash) | `b0c01649b9f0c8169c5d47b9c0177d37d44f56e04ab5bfab4192cfbee70909d7` | `830f1f7341de3d5ff2ba752e76a25db2c3a8bc180e72a5af8e026b5d16e20c96` |

The person pins move for two reasons at once, the new tree and the new
fixture keys chosen by device position. org-members'
`member_record_wire_bytes_are_pinned` stayed green unchanged. org-node's
`revocation_wire_message_is_pinned` was re-pinned to the value derived by
its format rule: the notice's proof is now the Vacant form at depth 256
(`00` Vacant, `8002` depth 256, `20` and a zero 32-byte map, `00` no
siblings, `00` Empty), the body length 119 → 123.

Coverage at T11 (stable rustc 1.99.0, cargo-llvm-cov 0.9.0, aarch64-darwin):
sparse-merkle-tree lines 98.80% (492 of 498), regions 96.59% (1049 of
1086), branches 77 of 78 (98.72%), with floors set at 97 / 95 / 97 and the
unreached code named beside them in the `Makefile`; person lines 100.00%
(458 of 458), regions 99.42% (684 of 688), branches 46 of 46; org-members
lines 96.22% (841 of 874), regions 93.88% (1320 of 1406), branches 93 of 94
(98.94%), each above its floors. person first measured below its floors
(98.69% / 98.55%) because `DeviceLeaf::new` and `DeviceLeaf::key` were never
called, and was closed by a test, not a lowered floor. CI lanes for the
new unit (test with and without default features, msrv, clippy, wasm32)
are in `.github/workflows/rust.yml`.

## Review

### Round 1

**finding-1**: code, low — the doc comment of `add_p2p_device` in org-members/src/trie.rs still said the bound was `MAX_DEVICES` (4); the bound is 8.
disposition: fixed — the doc comment now says `MAX_DEVICES` (8), org-members/src/trie.rs.

**finding-2**: code, low — org-members/README.md was out of date: security check 12 did not state what a Held proof carries, and the device-slot constraints did not include distinct device positions.
disposition: fixed — security check 12 lists what a Held proof carries (no DevicePublicKey; HAZ-r5tc4m, RC-5nka8z) and the device-slot constraints include distinct device positions (`DevicePositionTaken`), org-members/README.md.

**finding-3**: code, low — the inherited test `device_slot_order_does_not_change_the_root` did not test LLR-uudst2, which it was annotated to verify.
disposition: fixed — the test is deleted; LLR-uudst2 stays verified by `every_device_reaches_the_root` in org-members/tests/integration_test.rs.

**finding-4**: code, low — LLR-ggu9bh's upper bound (a depth above 256 does not compile) had no `compile_fail` doctest; only the depth-0 side did.
disposition: fixed — a `compile_fail` doctest for a depth of 257 sits beside the depth-0 one, sparse-merkle-tree/src/tree.rs.

## Guideline reviews

One review per guidelines file `guidelines-file.sh` printed for the change
diff; every path resolves to the installed defaults under
`.guardrails/templates/`. Findings are taken from the plan's "Guideline
reviews" section, where they are labelled per round (T-, C-, A- with the
round number); each block names the plan's label. Where the plan's label for
a finding of a group is not recorded one by one, the block says so. Where
the plan names no file for a fix, the file cited is one the fix commit on
this branch changed (`git log` of the round's fix commits, which name the
finding labels they address).

### .guardrails/templates/TEST_GUIDELINES.md

Governs: the test files the change adds or modifies in sparse-merkle-tree,
person, org-members and org-node (`sparse-merkle-tree/tests/`,
`person/tests/`, `org-members/tests/`, `org-members/quint/`,
`org-node/tests/encoding_golden.rs`).

### Round 1

**finding-1**: guideline, medium — Existing suites 1 — person/tests/crate_discipline.rs:57
disposition: fixed (plan T-1) — the in-place edit of `every_rejection_is_a_distinct_named_variant`, which carried no `verifies:` of its own, is reverted to master's list, and the five new variants' distinctness is checked in `the_proof_refusals_are_distinct_named_variants` (LLR-3dvfjb), person/tests/person_absence_proofs.rs.

**finding-2**: guideline, low — Seams 1 — sparse-merkle-tree/tests/store.rs:179-180
disposition: fixed (plan T-2) — the assertion that `calculate()` fills the receiver's cells, which no LLR states, is deleted from sparse-merkle-tree/tests/store.rs. The reviewer's note that two Quint runs sat under LLR-v6gfc7 is fixed too: they have their own LLR-5jh6t3 block in org-members/quint/membership.qnt.

### Round 2

**finding-1**: guideline, medium — Seams 1 — person/tests/crate_discipline.rs:144-166
disposition: fixed (plan T2-1) — the signature scan saw only `pub fn`/`pub type`/`pub use` lines; it now reads enum variants, pub fields and trait-impl methods, person/tests/support/public_signatures.rs, checked by `no_public_signature_names_the_tree_or_its_error`.

### Round 3

**finding-1**: guideline, medium — Seams 1 — person/tests/crate_discipline.rs
disposition: fixed (plan T3-1) — the scanner fixture test `public_signatures_are_read_whole` tested a test-local helper and carried no `verifies:`; its cases are folded into `no_public_signature_names_the_tree_or_its_error`, person/tests/crate_discipline.rs.

**finding-2**: guideline, medium — Seams 1 — org-members/tests/tree_error_mapping.rs
disposition: fixed (plan T3-2) — the fixture test `only_a_parameter_may_name_the_tree_error` is folded into `no_public_signature_returns_or_holds_the_tree_error`, org-members/tests/tree_error_mapping.rs.

### Round 4

**finding-1**: guideline, medium — Doubles 2/3 — person/tests/person_device_root.rs
disposition: fixed (plan T4-1) — person's copy of org-members' device hasher had no contract test; org-members' `hasher_domains_are_separated` now asserts `Blake3DeviceRootHasher`'s real domain keys and sentinel, `no_device_set_shares_its_root_with_the_person_device_root` compares the two real hashers (org-members/tests/integration_test.rs), and person's copy-based test is replaced by `no_domain_key_or_the_sentinel_is_org_members` (person/tests/person_device_root.rs).

**finding-2**: guideline, low — clause not recorded in the plan (an untested stated interface) — person/tests/device_root.rs
disposition: no change — (plan T4-2) `DeviceLeaf::new` and `public_key` are the public leaf of LLR-8y8tqr, exercised through that interface by `the_tree_holds_one_device_leaf_per_key`.

**finding-3**: guideline, low — clause not recorded in the plan (proptest seeds persisted, not printed) — person/tests/no_panic.rs
disposition: no change — (plan T4-3) proptest seeds are persisted, not printed, which is proptest's default and the workspace's practice.

**finding-4**: guideline, low — UI 6 — the template clause rather than the change
disposition: no change — (plan T4-4) the clause is the template's, not this project's; owner follow-up 2026-10-28 to tailor the guidelines.

### Round 5

**finding-1**: guideline, low — UI 6 — the template clause rather than the change
disposition: no change — the clause is for the guidelines-tailoring follow-up (owner, 2026-10-28). The reviewer's note was taken: the accepting case of `try_from_goes_through_parse` is restored in person/tests/slots.rs. TEST review done: nothing above low.

### .guardrails/templates/CODE_GUIDELINES.md

Governs: the source, test, model and build files the change adds or
modifies in sparse-merkle-tree, person, org-members and org-node, the root
`Makefile` and `.github/workflows/rust.yml`.

### Round 1

**finding-1**: guideline, low — Names 1/3 — short or generic names in tests
disposition: fixed (plan C-1) — descriptive names for short test parameters and locals, org-members/tests/fuzz_tests.rs, sparse-merkle-tree/tests/store.rs, person/tests/person_device_root.rs.

**finding-2**: guideline, low — Names 1/3 — short or generic names in src
disposition: fixed (plan C-2) — the formatter and the `TreeError` conversion's argument are named, org-members/src/error.rs.

**finding-3**: guideline, low — Names 1/3 — short names in Quint and generic parameters
disposition: no change — (plan C-3) Quint lambda parameters (`k`, `d`, `e`) and Rust generic type parameters (`L`, `H`, `D`) are the convention of the surrounding model and workspace; renaming only the new ones would split it.

**finding-4**: guideline, low — Names 1/3 — generic local names
disposition: fixed (plan C-4) — content bytes and JSON locals named for what they hold, org-members/src/hasher.rs.

**finding-5**: guideline, medium — Names 4 — org-members/src/trie.rs `apply_delta`
disposition: fixed (plan C-5) — `apply_delta` names the MemberTree `tree`, not `root`, org-members/src/trie.rs.

**finding-6**: guideline, low — clause not recorded in the plan (inline block expressions) — sparse-merkle-tree/tests/proofs.rs
disposition: fixed (plan C-6) — the last-byte position helper is named, sparse-merkle-tree/tests/proofs.rs.

**finding-7**: guideline, low — Simplicity 4 — the `keyed` helper duplicated in each crate's hasher adapter
disposition: no change — (plan C-7) `keyed` is private to each crate's hasher adapter and person exports no hashing helper; the test-local hashers are the oracle and stay independent of the code under test.

**finding-8**: guideline, low — Simplicity 4 — org-members' `calculated` duplicates the tree's rule
disposition: no change — (plan C-8) kept, with no new tree API; a comment says why `calculated` lives here, org-members/src/member_tree.rs.

**finding-9**: guideline, low — Simplicity 4 — the shared-position check repeated in parse, add and decode
disposition: fixed (plan C-9) — one `same_position` check, person/src/slots.rs.

**finding-10**: guideline, low — Simplicity 4 — MemberId position and identity hash computed twice
disposition: fixed (plan C-10) — one MemberId position and identity hash for both leaves, org-members/src/member_tree.rs.

**finding-11**: guideline, low — Simplicity 4 — test helpers duplicated
disposition: fixed (plan C-11) — one key-search helper and one positioned-key memo, org-members/tests/common/mod.rs.

**finding-12**: guideline, low — Comments — a commented-out old value
disposition: fixed (plan C-12) — removed; comments state what the pins are, org-node/tests/encoding_golden.rs, person/tests/person_hash.rs.

**finding-13**: guideline, low — Comments — change history in comments
disposition: fixed (plan C-13) — removed; the history is in the plan; comments state what the floors are, `Makefile`.

**finding-14**: guideline, low — Errors — a discarded error without a reason
disposition: fixed (plan C-14) — already met by the ARCHITECTURE round-1 fix, which says which `TreeError` variants reach each discarded error, person/src/absence.rs; one further comment added in sparse-merkle-tree/src/proof.rs.

**finding-15**: guideline, low — Errors — a discarded error without a reason
disposition: fixed (plan C-15) — a comment states why the discarded variants cannot occur, org-members/src/proof.rs.

**finding-16**: guideline, low — Errors — `from_siblings` skipped instead of refusing
disposition: fixed (plan C-16) — `from_siblings` returns `LevelOutOfRange` instead of skipping, sparse-merkle-tree/src/proof.rs.

**finding-17**: guideline, low — Errors — a discarded error without a reason in test support
disposition: fixed (plan C-17) — org-members/tests/common/mod.rs.

### Round 2

The plan names some of this round's findings by label (C2-2, C2-7, C2-8,
C2-9) and lists the rest by subject without labels; the blocks follow the
plan's order.

**finding-1**: guideline, low — Names 1/3 — Quint lambda names — org-members/quint/membership.qnt
disposition: fixed — Quint lambda names, org-members/quint/membership.qnt.

**finding-2**: guideline, low — Names 1/3 — generic parameter names (plan C2-2)
disposition: no change — repeats round-1 finding-3; the workspace convention for generic type parameters.

**finding-3**: guideline, low — Names — `value`/`how` locals
disposition: fixed — test-local names, sparse-merkle-tree/tests/proof_decoding.rs.

**finding-4**: guideline, low — Names — `ids()`
disposition: fixed — sparse-merkle-tree/tests/diff.rs.

**finding-5**: guideline, low — Names — the `device` field against the `device` parameter in org-members proof.rs
disposition: fixed — the field is `device_root_absence` after the later renames, org-members/src/proof.rs.

**finding-6**: guideline, low — Names — `leaf()`'s `name`/`index`
disposition: fixed — org-members/tests/fuzz_tests.rs.

**finding-7**: guideline, low — Simplicity 4 — the keyed helper (plan C2-7)
disposition: no change — repeats round-1 finding-7.

**finding-8**: guideline, low — Simplicity 4 — the test hashers (plan C2-8)
disposition: no change — repeats round-1 finding-7: the test-local hashers are the oracle.

**finding-9**: guideline, low — Simplicity 4 — `calculated` (plan C2-9)
disposition: no change — repeats round-1 finding-8.

**finding-10**: guideline, low — a hand-written `Debug` impl
disposition: no change — kept as dispositioned in the plan's round-2 fixes.

**finding-11**: guideline, low — Errors — the `unwrap_or` fallback in `depth()`
disposition: fixed — `Depth::<D>::AS_U16` replaces the fallback, sparse-merkle-tree/src/defaults.rs.

**finding-12**: guideline, low — Comments — a "now" in a Makefile comment
disposition: fixed — the note is in the present tense, `Makefile`.

### Round 3

**finding-1**: guideline, medium — Comments 2 — org-node/tests/encoding_golden.rs:126-134
disposition: fixed (plan C3-1) — the stale comment above `GOLDEN_REVOCATION` is rewritten, org-node/tests/encoding_golden.rs.

**finding-2**: guideline, medium — Simplicity 4 — the signature scanner copied into org-members' tests
disposition: fixed (plan C3-2) — one copy, person/tests/support/public_signatures.rs, reached by org-members' test through `#[path]`.

**finding-3**: guideline, low — Simplicity 4 — test helpers outside tests/common
disposition: fixed (plan C3-3) — person/tests/common/mod.rs.

**finding-4**: guideline, low — Simplicity 4 — test helpers outside tests/common
disposition: fixed (plan C3-4) — sparse-merkle-tree/tests/common/mod.rs.

**finding-5**: guideline, low — Simplicity 4 — test helpers outside tests/common
disposition: fixed (plan C3-5) — sparse-merkle-tree/tests/common/mod.rs.

**finding-6**: guideline, low — Simplicity 4 — test helpers outside tests/common
disposition: fixed (plan C3-6) — person/tests/common/mod.rs.

**finding-7**: guideline, low — repeat of a kept round-1 disposition (plan C3-7)
disposition: no change — kept as before (generic parameter names, keyed helpers, `calculated`, the `Debug` impl; see round 1 finding-3, finding-7, finding-8 and round 2 finding-10).

**finding-8**: guideline, low — a `NotHeldProof` alias
disposition: fixed (plan C3-8) — org-members/src/proof.rs.

**finding-9**: guideline, low — Names — the `group_key` accessor
disposition: fixed (plan C3-9) — resolved by ARCHITECTURE round 3 finding-5's rename to `person_key`, org-members/src/proof.rs.

**finding-10**: guideline, low — repeat of a kept disposition (plan C3-10)
disposition: no change — kept as before (see round 3 finding-7).

**finding-11**: guideline, low — repeat of a kept disposition (plan C3-11)
disposition: no change — kept as before (see round 3 finding-7).

**finding-12**: guideline, low — repeat of a kept disposition (plan C3-12)
disposition: no change — kept as before (see round 3 finding-7).

**finding-13**: guideline, low — Comments (plan C3-13)
disposition: fixed — person/src/device_root.rs.

**finding-14**: guideline, low — Comments (plan C3-14)
disposition: fixed — org-members/src/hasher.rs.

### Round 4

The plan names this round's fixed and kept findings by label (C4-3, C4-4,
C4-8, C4-9, C4-10, C4-12, C4-14, C4-15) and the repeats of kept
dispositions by subject; the remaining labels are assigned to those subjects
in the plan's order.

**finding-1**: guideline, low — Names 1/3 — generic parameter names
disposition: no change — repeats CODE round 1 finding-3.

**finding-2**: guideline, low — Simplicity 4 — keyed helpers
disposition: no change — repeats CODE round 1 finding-7.

**finding-3**: guideline, low — Names — `device_content` / `member_content`
disposition: fixed (plan C4-3) — `…_content_hash` names, org-members/src/hasher.rs, person/src/device_root_hasher.rs; LLR-9mjfjn and LLR-8y8tqr amended in place.

**finding-4**: guideline, low — Names — `p2p_key` on `MemberLeaf`
disposition: no change — (plan C4-4) `p2p_key` is master's `MemberLeaf` accessor, outside this change.

**finding-5**: guideline, low — Simplicity 4 — test hashers
disposition: no change — repeats CODE round 1 finding-7.

**finding-6**: guideline, low — Simplicity 4 — `calculated`
disposition: no change — repeats CODE round 1 finding-8.

**finding-7**: guideline, low — the `Debug` impl
disposition: no change — repeats CODE round 2 finding-10.

**finding-8**: guideline, low — clause not recorded in the plan — org-members domain keys inline
disposition: fixed (plan C4-8) — the domain keys are named constants, org-members/src/hasher.rs.

**finding-9**: guideline, low — Simplicity 4 — the src walk duplicated beside the shared scanner
disposition: fixed (plan C4-9) — the src walk is in person/tests/support/public_signatures.rs.

**finding-10**: guideline, low — Simplicity — the `device` wrapper in absence_proofs
disposition: fixed (plan C4-10) — removed, org-members/tests/absence_proofs.rs.

**finding-11**: guideline, low — Comments — the org-node golden history
disposition: no change — the dated history beside the golden values is that file's convention, org-node/tests/encoding_golden.rs.

**finding-12**: guideline, low — Comments — ruling dates beside bounds
disposition: no change — (plan C4-12) ruling dates are the project's convention for bounds.

**finding-13**: guideline, low — Comments — dated lines in the Makefile and rust.yml
disposition: no change — dated lines are those files' convention, `Makefile`, `.github/workflows/rust.yml`.

**finding-14**: guideline, low — Comments — `LevelOutOfRange` without a reason
disposition: fixed (plan C4-14) — a comment states when `LevelOutOfRange` occurs, org-members/src/error.rs.

**finding-15**: guideline, low — Comments — `bit` cites no LLR
disposition: fixed (plan C4-15) — `bit` cites LLR-mm45ry, sparse-merkle-tree/src/bits.rs. CODE review done: nothing above low.

### .guardrails/templates/ARCHITECTURE_GUIDELINES.md

Governs: the source and design files the change adds or modifies in
sparse-merkle-tree, person and org-members (`*/src/`, `*/docs/architecture/`,
`*/docs/CONTEXT.md`, `docs/CONTEXT.md`, `docs/adr/`), and
`.guardrails/units.yaml`.

The ARCHITECTURE review ended by owner ruling after round 8 (2026-10-28):
fix the round-8 medium, run no further rounds, and fold the lows into the
ARCHITECTURE guidelines-tailoring follow-up. The standing owner rulings the
later rounds cite are those of 2026-10-28: (1) Boundaries 2 — a mapped error
keeps no cause and the tree's proof types are shared by design; (2) serde
derives on core types plus one wire module per crate; (3) absence-proof
naming in the glossary.

### Round 1

**finding-1**: guideline, medium — Boundaries 2 — person exports `device_tree`/`DeviceTree`, whose methods return `TreeError` — person/src/device_root.rs
disposition: fixed (plan A-1) — owner ruling 2026-10-27: `prove_device_absent_in::<H>(devices, key)` returning `Result<AbsenceProof<4>, IdentityError>` replaces the export; LLR-8y8tqr amended; `no_public_signature_names_the_tree_or_its_error`, person/tests/crate_discipline.rs.

**finding-2**: guideline, low — Boundaries 2 — `map_err(|_| …)` discards the cause — person/src/absence.rs
disposition: fixed (plan A-2) — comments say which `TreeError` variants reach each discarded error, person/src/absence.rs, person/src/device_root.rs.

**finding-3**: guideline, medium — Domain 3 — "position" names both the 32 bytes and the place — sparse-merkle-tree/src/leaf.rs
disposition: fixed (plan A-3) — owner ruling 2026-10-27: the bytes are renamed `Leaf::position_bytes`, `device_position_bytes`, sparse-merkle-tree/src/leaf.rs; glossary gains "Position bytes", docs/CONTEXT.md.

**finding-4**: guideline, medium — Domain 2 — undefined public terms ProofEnding, identity hash, content hash, empty-subtree hash — sparse-merkle-tree/src/proof.rs
disposition: fixed (plan A-4) — glossary entries, sparse-merkle-tree/docs/CONTEXT.md.

**finding-5**: guideline, medium — Domain 2 — undefined public terms ProvenMember, Vacant/Held proof, device absence — org-members/src/proof.rs
disposition: fixed (plan A-5) — glossary entries, org-members/docs/CONTEXT.md.

**finding-6**: guideline, medium — Domain 1 — "device key" in user-facing text — person/src/slots.rs
disposition: fixed (plan A-6) — DevicePublicKey in new user-facing text; the remainder (the `DeviceSlots` `expecting` text, quoted by LLR-5za6mp) by LLR-cyx9a3, person/src/slots.rs.

**finding-7**: guideline, low — Domain 1 — `TrieHasher` keeps "trie" — org-members/src/hasher.rs
disposition: fixed (plan A-7) — renamed `MemberHasher`, org-members/src/hasher.rs.

**finding-8**: guideline, medium — Core and edges 4 — `device_position` calls blake3 outside person's hasher adapter — person/src/device_root.rs
disposition: fixed (plan A-8) — the call moved into the adapter; blake3 is now only in person/src/keyed_hash.rs.

**finding-9**: guideline, low — Core and edges 4 — serde in several modules
disposition: no change — (plan A-9) derives plus one wire module per crate is the workspace's existing pattern; later accepted by owner ruling (2) of 2026-10-28.

**finding-10**: guideline, medium — Boundaries 3 — an SDD statement with two responsibilities — sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md
disposition: fixed (plan A-10) — one-sentence statement, SDD-x43d2z ("Consumer interfaces").

**finding-11**: guideline, medium — Boundaries 3 — an SDD statement with two responsibilities — sparse-merkle-tree/docs/architecture/2026-10-24-sparse-merkle-tree.md
disposition: fixed (plan A-11) — one-sentence statements, SDD-rh9u6z, SDD-ur6uvu.

**finding-12**: guideline, medium — Boundaries 3 — an SDD statement with two responsibilities — person/docs/architecture/2026-10-24-device-root-tree.md
disposition: fixed (plan A-12) — one-sentence statements, SDD-bx9mm6, SDD-7r833z.

**finding-13**: guideline, low — Boundaries 3 — a stale SDD statement — org-members/docs/architecture/2026-09-17-decomposition.md
disposition: fixed (plan A-13) — SDD-d9svdj restated.

**finding-14**: guideline, medium — Constraints 1 — no architecture test for "no TreeError crosses the interface" — person/tests/crate_discipline.rs
disposition: fixed (plan A-14) — `no_public_signature_names_the_tree_or_its_error`, person/tests/crate_discipline.rs.

### Round 2

**finding-1**: guideline, medium — Boundaries 2 (cause not kept) — org-members/src/error.rs `From<TreeError>`, person/src/device_root.rs, person/src/absence.rs
disposition: no change — (plan A2-1) recurred after round 1; owner ruling 2026-10-28: accepted as designed — a mapped error names the check and keeps no cause; LLR-3dvfjb and LLR-w5yahk scope "no TreeError crosses" to each unit's own functions.

**finding-2**: guideline, medium — Boundaries 2 (dependency's error type reaches callers) — person `PersonAbsenceProof`, `prove_device_absent_in`; org-members `vacancy()`, `inclusion()`, `device_absence()` — person/src/absence.rs, org-members/src/proof.rs
disposition: no change — (plan A2-2) owner ruling 2026-10-28: the sparse-merkle-tree unit is a declared dependency of both consumers and its proofs are shared by design (docs/adr/2026-10-23-one-sparse-merkle-tree.md).

**finding-3**: guideline, medium — Domain 3 — "absence proof" means two things in org-members — org-members/src/proof.rs
disposition: fixed (plan A2-3) — org-members/src/proof.rs; LLR-v6smww, LLR-bh53rr reworded.

**finding-4**: guideline, medium — Domain 2 — `PersonAbsenceProof` not in a glossary — person/docs/CONTEXT.md
disposition: fixed (plan A2-4) — "Person absence proof" in person/docs/CONTEXT.md.

**finding-5**: guideline, medium — Domain 2 — calculated, level, map, siblings, node hash, empty leaf not in the tree unit's glossary — sparse-merkle-tree/docs/CONTEXT.md
disposition: fixed (plan A2-5) — glossary entries Calculated, Level, Node hash, Empty leaf, Sibling, Proof map, sparse-merkle-tree/docs/CONTEXT.md.

**finding-6**: guideline, medium — Domain 1 — `ProvenMember::p2p_key()` uses an avoided term — org-members/src/proof.rs
disposition: fixed (plan A2-6) — renamed (to `person_key` after ARCHITECTURE round 3 finding-5), org-members/src/proof.rs.

**finding-7**: guideline, low — Domain 1 — "member key" in new prose
disposition: fixed (plan A2-7) — docs/CONTEXT.md, org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md.

**finding-8**: guideline, low — Domain 1 — "trie" in new prose
disposition: fixed (plan A2-8) — org-members/src/trie.rs, org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md.

**finding-9**: guideline, low — Domain 1 — bare "key" in new prose
disposition: fixed (plan A2-9) — "leaf identity" for the tree's side, sparse-merkle-tree/docs/CONTEXT.md.

**finding-10**: guideline, low — Core and edges 4 — blake3 in two person modules
disposition: fixed (plan A2-10) — blake3 only in the private person/src/keyed_hash.rs; position bytes in person/src/position.rs.

**finding-11**: guideline, low — Core and edges 4 — serde spread
disposition: no change — (plan A2-11) kept; accepted by owner ruling (2) of 2026-10-28.

**finding-12**: guideline, low — Boundaries — the store builds proofs from `pub(crate)` fields — sparse-merkle-tree/src/tree.rs
disposition: no change — (plan A2-12) kept as dispositioned in the plan's round-2 fixes; the fields are `pub(crate)`, not public, sparse-merkle-tree/src/tree.rs.

**finding-13**: guideline, low — Boundaries 3 — device_hasher.rs's two jobs
disposition: fixed (plan A2-13) — the module holds the Person device root hasher alone, now person/src/device_root_hasher.rs.

**finding-14**: guideline, low — Constraints 1 — no org-members architecture test for LLR-w5yahk's structural half
disposition: fixed (plan A2-14) — `no_public_signature_returns_or_holds_the_tree_error`, org-members/tests/tree_error_mapping.rs.

### Round 3

**finding-1**: guideline, medium — Boundaries 2 (cause) (plan A3-1)
disposition: no change — owner ruling (1) of 2026-10-28 stands: accepted.

**finding-2**: guideline, medium — Boundaries 2 (cause) (plan A3-2)
disposition: no change — owner ruling (1) of 2026-10-28 stands: accepted.

**finding-3**: guideline, medium — Core and edges 4 — serde spread; recurs A-9/A2-11 (plan A3-3)
disposition: no change — owner ruling 2026-10-28: accepted for this change (derives on core types plus one `wire` module per crate is the workspace pattern); follow-up to tailor the ARCHITECTURE guidelines to allow serde derives.

**finding-4**: guideline, low — Core and edges — spin used in one place (plan A3-4) — sparse-merkle-tree/src/node.rs
disposition: no change — kept: `spin` is used in one place, sparse-merkle-tree/src/node.rs.

**finding-5**: guideline, medium — Domain 1 — `ProvenMember::group_key()`: "group key" is on Member-as-a-group key's Avoid list (plan A3-5) — org-members/src/proof.rs
disposition: fixed — `person_key()`, the code name docs/CONTEXT.md gives a PersonPublicKey, org-members/src/proof.rs.

**finding-6**: guideline, low — Domain 1 — `DeviceLeaf::key` (plan A3-6) — person/src/device_root.rs
disposition: fixed — `DeviceLeaf::public_key`, person/src/device_root.rs.

**finding-7**: guideline, low — Domain 1 — `device_identity`/`member_identity` (plan A3-7)
disposition: fixed — `…_identity_hash`, org-members/src/hasher.rs, person/src/device_root_hasher.rs.

**finding-8**: guideline, low — Domain 1 — avoided words in new prose (plan A3-8)
disposition: fixed — org-members/docs/architecture/2026-10-24-on-sparse-merkle-tree.md.

**finding-9**: guideline, low — Domain 1 — avoided words in new prose (plan A3-9)
disposition: fixed — person/docs/architecture/2026-10-24-device-root-tree.md.

**finding-10**: guideline, low — Domain 1 — avoided words in new prose (plan A3-10)
disposition: fixed — org-members/docs/CONTEXT.md.

**finding-11**: guideline, low — Domain 2 — Leaf and Hasher not in the glossary (plan A3-11)
disposition: fixed — glossary Leaf and Tree hasher, sparse-merkle-tree/docs/CONTEXT.md.

**finding-12**: guideline, low — Constraints 1 — the module doc's claim "only keyed_hash calls blake3" has no LLR or test (plan A3-12)
disposition: fixed — person/src/keyed_hash.rs.

### Round 4

The plan gives this round's severities as a total and lists A4-5..A4-7 as a
group; the medium/low split among them below follows the plan's order.

**finding-1**: guideline, medium — Boundaries 2 (cause) (plan A4-1)
disposition: no change — owner ruling (1) of 2026-10-28 stands: accepted.

**finding-2**: guideline, medium — Boundaries 2 (cause) (plan A4-2)
disposition: no change — owner ruling (1) of 2026-10-28 stands: accepted.

**finding-3**: guideline, medium — Boundaries 2 (cause) (plan A4-3)
disposition: no change — owner ruling (1) of 2026-10-28 stands: accepted.

**finding-4**: guideline, medium — Domain 3 — "absence proof" names two types in org-members; recurs A2-3 (plan A4-4)
disposition: fixed — owner ruling 2026-10-28: the glossary makes "Absence proof" the general term, with Tree, Device and Person absence proofs and Vacant and Held as the two Device absence proofs; no code change, docs/CONTEXT.md.

**finding-5**: guideline, medium — Core and edges 4 — serde spread (plan A4-5)
disposition: no change — owner ruling (2) of 2026-10-28 stands: accepted.

**finding-6**: guideline, medium — Core and edges 4 — serde spread (plan A4-6)
disposition: no change — owner ruling (2) of 2026-10-28 stands: accepted.

**finding-7**: guideline, low — Core and edges 4 — serde spread (plan A4-7)
disposition: no change — owner ruling (2) of 2026-10-28 stands: accepted.

**finding-8**: guideline, low — Domain 2 — Member hasher and Device root hasher not in a glossary (plan A4-8)
disposition: fixed — glossary entries, org-members/docs/CONTEXT.md.

**finding-9**: guideline, low — Domain 2 — Empty-leaf sentinel not in a glossary (plan A4-9)
disposition: fixed — glossary entry, docs/CONTEXT.md.

### Round 5

**finding-1**: guideline, medium — Domain 3 — `device_absence()` / `Held { device_absence }` hold a Tree absence proof while the glossary's Device absence proof is the whole proof; recurs A4-4 at code level (plan A5-1) — org-members/src/proof.rs
disposition: fixed — owner ruling 2026-10-28: renamed `device_root_absence`, type names unchanged, org-members/src/proof.rs.

**finding-2**: guideline, medium — Domain 1 — `Blake3DeviceHasher` and `MemberHasher::Device` use the avoided "device hasher" / bare "Device" (plan A5-2) — org-members/src/hasher.rs
disposition: fixed — `Blake3DeviceRootHasher` and the associated type renamed (finally `DeviceRootHasher`, round 8), org-members/src/hasher.rs.

**finding-3**: guideline, medium — Boundaries 3 — SDD-d6x85b's statement (two responsibilities, both now false), its heading and README row; lesser, SDD-57vaj4 (plan A5-3)
disposition: fixed — SDD-d6x85b and SDD-57vaj4 restated in one sentence each, org-members/docs/architecture/2026-10-06-absence-proofs.md, org-members/docs/architecture/README.md.

**finding-4**: guideline, low — Domain 1 — `empty_hash` (plan A5-4)
disposition: fixed — `empty_subtree_hash`, named in LLR-yuyn8f, sparse-merkle-tree/src/tree.rs.

**finding-5**: guideline, low — Domain 2 — Device leaf and Device position not in a glossary (plan A5-5)
disposition: fixed — glossary entries, person/docs/CONTEXT.md.

**finding-6**: guideline, low — under owner ruling (1) (plan A5-6)
disposition: no change — owner ruling (1) of 2026-10-28: accepted.

**finding-7**: guideline, low — under owner ruling (2) (plan A5-7)
disposition: no change — owner ruling (2) of 2026-10-28: accepted.

**finding-8**: guideline, low — spin in node.rs (plan A5-8) — sparse-merkle-tree/src/node.rs
disposition: no change — kept, as round 3 finding-4.

**finding-9**: guideline, low — `same_device_position` and LLR-vraz7n belong to the DeviceSlots item or the device root's stated interface (plan A5-9)
disposition: fixed — `same_device_position` stated in LLR-exwx4n as the device root's comparison of two keys' device positions, person/docs/architecture/2026-10-24-device-root-tree.md.

### Round 6

**finding-1**: guideline, medium — Domain 3 — "device root" names both the value and the tree (plan A6-1)
disposition: fixed — "Device root" keeps the value and "Device root tree" names the tree in the glossaries; every changed line meaning the tree says so, including SDD-7r833z, LLR-uudst2, LLR-utk6uy, LLR-5jh6t3, LLR-exwx4n, REQ-9sh8qb, REQ-4e23xr, REQ-2sz7j8, HAZ-vjm5fg, person/src/device_root.rs.

**finding-2**: guideline, low — Domain 1 — `MemberId` doc says "p2p key" (plan A6-2) — org-members/src/types.rs
disposition: fixed — the doc names the Member-as-a-group key, org-members/src/types.rs.

**finding-3**: guideline, low — Domain 2 — "vacancy" undefined (plan A6-3)
disposition: fixed — the glossary ties it to the Vacant proof's Tree absence proof, org-members/docs/CONTEXT.md.

**finding-4**: guideline, low — spin in node.rs (plan A6-4) — sparse-merkle-tree/src/node.rs
disposition: no change — kept, as round 3 finding-4. The reviewer's notes were fixed too: the stale `spin::Once` wording in org-members/src/trie.rs and the units.yaml comment on the edges, .guardrails/units.yaml.

### Round 7

**finding-1**: guideline, medium — Domain 1/2 — `SparseMerkleTree::diff` and "upserted" undefined, and "diff" is on Change set's Avoid list (plan A7-1) — sparse-merkle-tree/src/tree.rs
disposition: fixed — renamed `changes_between`; glossary "Tree changes" and "Upserted leaf"; LLR-u2r4vj amended, sparse-merkle-tree/src/tree.rs.

**finding-2**: guideline, medium — Domain 1 — person's `PersonDeviceHasher` / module `device_hasher` use the avoided "device hasher" (plan A7-2)
disposition: fixed — `PersonDeviceRootHasher` in module person/src/device_root_hasher.rs.

**finding-3**: guideline, low — Domain 1 — `PersonAbsenceProof::absence()` (plan A7-3) — person/src/absence.rs
disposition: fixed — `device_root_absence()`, as org-members' Held accessor, person/src/absence.rs.

**finding-4**: guideline, low — under the standing owner rulings (plan A7-4)
disposition: no change — owner rulings (1)–(3) of 2026-10-28: accepted.

**finding-5**: guideline, low — serde field refusals and the standing rulings (plan A7-5)
disposition: no change — the serde-field part is required by LLR-9g2kjg and LLR-985v9t (every decode refusal is the one malformed-proof variant); the rest under owner rulings of 2026-10-28: accepted.

**finding-6**: guideline, low — under the standing owner rulings (plan A7-6)
disposition: no change — owner rulings (1)–(3) of 2026-10-28: accepted.

**finding-7**: guideline, low — under the standing owner rulings (plan A7-7)
disposition: no change — owner rulings (1)–(3) of 2026-10-28: accepted. The reviewer's notes were fixed too: org-members' SOUP rows for `person` and for sparse-merkle-tree (a unit dependency, not SOUP), org-members/docs/architecture/soup.md, and SDD-bx9mm6's title "Person absence proof".

### Round 8

**finding-1**: guideline, medium — Domain 1/3 — `MemberHasher::DeviceRoot` names a hasher with the term for the root value; recurs A5-2 (plan A8-1) — org-members/src/hasher.rs:11
disposition: fixed — owner ruling 2026-10-28: renamed `type DeviceRootHasher: person::DeviceRootHasher`, org-members/src/hasher.rs; then end the ARCHITECTURE reviews.

**finding-2**: guideline, low — Domain — the `leaf_hash` term (plan A8-2)
disposition: no change — owner ruling 2026-10-28: no further rounds; folded into the ARCHITECTURE guidelines-tailoring follow-up.

**finding-3**: guideline, low — Domain 3 — "depth" for one node's distance in the store's internals (plan A8-3) — sparse-merkle-tree/src/tree.rs
disposition: no change — owner ruling 2026-10-28: folded into the ARCHITECTURE guidelines-tailoring follow-up (internal names).

**finding-4**: guideline, low — Boundaries 3 — SDD-7r833z understates its LLRs (plan A8-4)
disposition: no change — owner ruling 2026-10-28: folded into the ARCHITECTURE guidelines-tailoring follow-up.

**finding-5**: guideline, low — serde field refusals mapped to the malformed-proof variant (plan A8-5)
disposition: no change — LLR-985v9t and LLR-9g2kjg require it; owner ruling 2026-10-28: no further rounds.

**finding-6**: guideline, low — under owner rulings (1)–(3) or Existing code 1 (one of the plan's group A8-6..A8-10)
disposition: no change — owner ruling 2026-10-28: no further rounds; folded into the ARCHITECTURE guidelines-tailoring follow-up.

**finding-7**: guideline, low — under owner rulings (1)–(3) or Existing code 1 (one of the plan's group A8-6..A8-10)
disposition: no change — owner ruling 2026-10-28: no further rounds; folded into the ARCHITECTURE guidelines-tailoring follow-up.

**finding-8**: guideline, low — under owner rulings (1)–(3) or Existing code 1 (one of the plan's group A8-6..A8-10)
disposition: no change — owner ruling 2026-10-28: no further rounds; folded into the ARCHITECTURE guidelines-tailoring follow-up.

**finding-9**: guideline, low — under owner rulings (1)–(3) or Existing code 1 (one of the plan's group A8-6..A8-10)
disposition: no change — owner ruling 2026-10-28: no further rounds; folded into the ARCHITECTURE guidelines-tailoring follow-up.

**finding-10**: guideline, low — under owner rulings (1)–(3) or Existing code 1 (one of the plan's group A8-6..A8-10)
disposition: no change — owner ruling 2026-10-28: no further rounds; folded into the ARCHITECTURE guidelines-tailoring follow-up.

## Gaps

- The app's npm verify entries (`npm --prefix app run check`, `run test`)
  were not run: app/node_modules is absent in the worktrees
  (`svelte-kit: command not found`, exit 127). Owner ruling 2026-10-28: the
  app is untouched beyond `app/src-tauri/Cargo.lock`, which records the new
  path dependencies with no version changed; its cargo entry passed.
- The msrv (1.85) CI lane added for the new unit was not run locally.
- The ARCHITECTURE review was ended by owner ruling after round 8; its
  remaining low findings are not fixed but folded into a follow-up to
  tailor the project's ARCHITECTURE guidelines (serde derives, root against
  unit glossary in a multi-unit repository, internal names), with the
  template's UI 6 clause in TEST as a separate tailoring point. Round 6
  also raised findings under the standing rulings that the plan records
  only as a group, not one by one; they are accepted under those rulings.
- The Boundaries 2 findings (a mapped error keeps no cause; the tree's
  proof types reach consumers' callers) and the serde-spread findings are
  accepted by owner ruling 2026-10-28, not fixed.
- on-chain-client's `problem_age_days` was raised from 45 to 90 by owner
  ruling 2026-10-28 (`on-chain-client/.guardrails/config.yaml`), because
  that unit's open problem items reached the old limit during this change;
  this change resolves none of them.
- Red-first evidence is weaker than a failed assertion in these places,
  each named in its Red → green row: most tests were red only by compile
  failure of the missing API; org-members' T7 tests were red only as a
  library compile failure, not on their own assertions; the Quint tests
  `addDevicePositionTakenTest`, `addDevicePositionBeforeDuplicateKeyTest`
  and `scenarioDevicePositionTaken` were not watched red; the fuzz
  properties `every_honest_absence_proof_verifies` (Held case) and
  `absence_proof_decoding_and_verify_never_panic` (new `arb_wire_proof`)
  were not run before the implementation;
  `any_operation_sequence_does_not_panic` was not red; tests added after
  the behaviour existed (T11, the review fixes) were red only against
  mutants; the depth-257 `compile_fail` doctest was not watched red.
- `cargo fmt --check` reports diffs in the new crate; formatting is not a
  gate in this repository.
