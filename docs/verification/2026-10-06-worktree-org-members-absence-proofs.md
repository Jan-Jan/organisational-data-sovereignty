# Verification — absence proofs, S1 (2026-10-06)

branch: worktree-org-members-absence-proofs
reviewer: independent review subagent (fresh context, given only the diff, the plan and `find-items.sh show` of each claimed ID; it ran the suite in its own review worktree), 2026-10-06
verdict: approve. The proof math is sound: path order, default-map layout, fold direction, no proof verifies for a present device or member, memory is bounded, no panic is reachable from input. Seven low or record findings, all fixed in this change. The review round was the last one, because every code and requirement finding was low.
reproduced: yes, for PR-jq43gx. Before the fix, `member_id_bit_out_of_range_does_not_panic` panicked with "index out of bounds: the len is 32 but the index is 32" (types.rs:58), and `default_hashes_at_level_out_of_range_does_not_panic` panicked with "the len is 257 but the index is 257" (smt.rs:35). The absence proofs are new capability, not a defect, so they had nothing to reproduce. Each test was watched red, as the table below records.

Change: org-members gains absence proofs. Any device holding the current members trie can prove that a MemberId holds no Member, or that the Member it holds lacks a Device key, and a revoked device can verify that proof against the on-chain root. `MemberId::bit` and `DefaultHashes::at_level` no longer panic. Branched from `master` at `0de6586`, with local master merged in at `fdf4e77` and then at `273f0e3`.
Plan: `docs/plans/2026-10-06-absence-proofs.md`. Roadmap: `docs/plans/2026-10-06-org-io-roadmap.md` (stage S1).

Units: org-members touched; org-node touched (documentation only: a new problem report); app dependent. The impact set is from `check-units.sh --impact refs/heads/master..HEAD`.

## The gate

Measured on: `f041aba` (`git rev-parse HEAD`), tree `cde106eb1913811889edfcbba55095a7698ff5c4` (`git rev-parse HEAD^{tree}` on a clean worktree). This was round 2, step 2. Step 3 renamed nothing in this round, so the step 2 gate summary stands. Log: the session scratchpad's `s1-merge-gate4.log`.

| Gate | Result |
| --- | --- |
| `cargo test -p org-members` | 271 passed, 0 failed, 1 ignored (`preflight_probe`, ignored by design) |
| `quint` version, 3 typechecks, `quint test membership.qnt` | ok; 63 passing |
| `quint run membership_mbt.qnt --invariant=mbtInv` | no violation found |
| `cargo clippy -p org-members --all-targets -- -D warnings` | clean |
| `cargo build -p org-members --no-default-features` | clean |
| org-node `cargo test` (25 targets) | 228 passed, 0 failed; the 3 fuzz targets ran their full budget without a failure |
| org-node quint typechecks | ok |
| org-node 5 `quint run` invariants | **not run**, by owner ruling of 2026-10-06: org-node is touched only by a problem-report document |
| app `cargo test` (10 targets) | 159 passed, 0 failed |
| app `npm run check` | 0 errors, 1 warning: "Cannot find type definition file for 'node'". This comes from the app's locked dependency set on master (no `@types/node` in the lockfile), not from this change |
| app `npm run test` | 43 passed |
| `check-ids.sh` (org-members, org-node, app; no draft flag) | clean |
| `check-trace.sh` (org-members, org-node, app) | exit 0. This change resolves PR-jq43gx (org-members) and opens PR-qmvj83 (org-node) |
| `check-units.sh` | exit 0 |
| Coverage, `make coverage-org-members` (stable) | lines 96.55%, regions 94.57%; floors 92 / 91 met. `absence_proofs` is now in the measured set |
| Decision coverage, nightly `--branch` | 123 of 124 branches. Every branch in changed code is covered: proof.rs 16/16, smt.rs 20/20, types.rs 16/16. The one miss is trie.rs:449 (`update_leaf`), which this change did not touch *Corrected 2026-10-06 (worktree-guardrails-branch-coverage review): the miss is the `!old.is_calculated()` operand of `calculate_delta`'s guard, not `update_leaf`, which has no counted branch.* |
| Working tree | clean |

## Red → green

| Item | Test | Watched red |
| --- | --- | --- |
| LLR-7jkcba, PR-jq43gx | `member_id_bit_out_of_range_does_not_panic`, `default_hashes_at_level_out_of_range_does_not_panic` | panicked (index out of bounds) before the fix |
| LLR-7jkcba | `out_of_range_path_index_is_index_out_of_range` | could not compile before the fix (it names `IndexOutOfRange`); its red evidence is the two reproductions above |
| LLR-4xz255, REQ-535jcd | `prove_absent_refuses_an_uncalculated_trie`, `prove_absent_refuses_a_device_the_member_holds`, `prove_absent_ignores_a_key_another_member_holds` | E0599, no method `prove_absent` |
| LLR-4xz255, REQ-535jcd | `prove_absent_refuses_an_uncalculated_trie_even_on_a_hashed_path` | failed (got `Ok`) with the `is_calculated()` guard deleted |
| LLR-25tpdp, REQ-yyuxh8 | `proof_for_an_absent_member_carries_no_member_data`, `proof_for_a_removed_device_carries_only_that_member` | E0432/E0599, `org_members::proof` and `prove_absent` missing |
| LLR-25tpdp, REQ-yyuxh8, RC-qa2758 | `a_proof_next_to_a_neighbour_carries_none_of_its_data` | failed with "proof bytes carry the neighbour's handle" under mutation M4 (`path` returns the neighbouring sibling's leaf) |
| LLR-25tpdp, LLR-utp6x4 | `a_proof_sends_only_non_default_siblings` | failed with every level sent explicitly (map all 0x00), and with the map bit at `7 - level % 8` (last byte 0xfe) |
| LLR-dgzy7e, LLR-p2p8qy, LLR-utp6x4, REQ-tk2qqj, RC-j2znx8 | `proof_for_an_absent_member_verifies`, `proof_for_a_deleted_member_verifies`, `proof_for_a_removed_device_verifies`, `proof_is_refused_against_another_root`, `proof_is_refused_for_another_member_id`, `proof_is_refused_when_the_member_holds_the_device` | E0599, no method `verify` |
| LLR-utp6x4 | `proofs_at_the_lowest_and_highest_ids_verify_when_absent`, `proofs_at_the_lowest_and_highest_ids_verify_for_a_removed_device`, `proofs_in_a_one_member_organisation_verify` | failed with `AbsenceProofRootMismatch` under M5a and M5b (one arm of `path`'s sibling choice swapped) |
| LLR-4rju5r, LLR-dgzy7e | `from_parts_refuses_a_count_that_does_not_match_the_map`, `a_tampered_sibling_is_refused_by_root` | E0599, no `from_parts` |
| LLR-4rju5r, REQ-tk2qqj | `a_proof_survives_the_wire_and_still_verifies`, `the_wire_refuses_inconsistent_or_oversized_proofs` | E0277, `AbsenceProof` was not `Serialize`/`Deserialize` |
| LLR-4rju5r, REQ-ds8ryr | `the_wire_stops_reading_siblings_at_the_bound` | `DeserializeUnexpectedEnd` with the visitor's 256 bound deleted; `SerdeDeCustom` with it in place |
| LLR-4rju5r, REQ-tk2qqj | `an_empty_ending_proof_survives_the_wire_and_still_verifies` | failed under M3 (the wire refused an `Empty` ending) |
| REQ-ds8ryr, LLR-2dcnbp, LLR-4rju5r | `absence_proof_decoding_and_verify_never_panic` | caught a planted index panic in `verify` (proof.rs:136) once structured proofs were generated |
| LLR-utp6x4, LLR-dgzy7e, LLR-4xz255, REQ-535jcd, REQ-tk2qqj | `every_honest_absence_proof_verifies` | failed with `AbsenceProofRootMismatch` under `==` → `!=` in `from_path` |

The plan's Progress table holds these attestations task by task, as each dispatch reported them.

## What was wrong, and what was built

**PR-jq43gx.** `MemberId::bit` indexed a `[u8; 32]` with `index / 8`, and `DefaultHashes::at_level` indexed a 257-element `Vec`, both unchecked. Both are public, and both panicked on values their `u16` argument admits. They now return `IndexOutOfRange`. The store's own walks steer with `MemberId::path_bits`, an iterator over the 256 bits that cannot go out of range. Without it, `contains` and `get` would have had to return `Result`, which would break org-node and the app.

**Absence proofs** (SDD-57vaj4, `org-members/src/proof.rs`). A proof always covers 256 levels. A 32-byte map marks the siblings that equal their level's default hash; only the other siblings are sent. The proof ends in `Empty` or in the leaf of the Member under the proof's MemberId.
- `OrgTrie::prove_absent` refuses an uncalculated trie, and refuses a device that the Member still holds.
- `AbsenceProof::verify` recomputes the root along the MemberId's path, refuses a mismatch first, then refuses a leaf that still holds the device.
- `from_parts` and the bounded serde form are the parsing edge.

The revoked-device record, and the gating of who may receive a proof, belong to org-io and org-node (roadmap S3 and S5). The README's section 12 states the caller's duties.

**PR-qmvj83** is opened, not fixed. A revocation today sends the revoked device every member's record (LLR-8hdu9x). The fix is roadmap S3.

## Review

**finding-1**: code, low — The default-sibling compression and the proof's wire layout were not pinned by any test. Every count assertion checked `siblings().len() + default_level_count(map) == 256`, which also holds if `from_path` sent all 256 siblings explicitly. The map layout and the leaf-up sibling order were held only by `from_path` and `verify` sharing one helper.
disposition: added `a_proof_sends_only_non_default_siblings`. It checks the exact map bytes and the number of explicit siblings in a one-member organisation and in a two-neighbour organisation, and it confirms against a separately computed root that `siblings()[0]` is the neighbour's leaf hash. It reddens with every level sent explicitly, and with the map bit written at `7 - level % 8`.

**finding-2**: code, low — `prove_absent_refuses_an_uncalculated_trie` could not detect removal of the `is_calculated()` guard, because the walk itself hits the unhashed, freshly path-copied branch.
disposition: added `prove_absent_refuses_an_uncalculated_trie_even_on_a_hashed_path`. It removes alice's device without recalculating, then asks for alice's own path, all of whose siblings are hashed. It reddens with the guard deleted, while the older test stays green.

**finding-3**: requirement, low — LLR-4rju5r and LLR-2dcnbp claimed that serde parsing refuses with the typed `AbsenceProofMalformed`. A postcard caller in fact receives `postcard::Error::SerdeDeCustom`.
disposition: both LLRs were reworded in `org-members/docs/architecture/2026-10-06-absence-proofs.md`. `from_parts` returns the typed variant; through a serde format, the variant arrives as that format's custom error, carrying the variant's message. The same edit made LLR-4rju5r's memory statement agree with finding-4. IDs are unchanged.

**finding-4**: record — The risk file said memory use does not depend on the input, but a Leaf ending's strings are decoded before validation.
disposition: `org-members/docs/risk/2026-10-06-absence-proofs.md`, "Hostile input", now says the sibling list is bounded at 256 while decoding, and that a Leaf ending's handle, name and surname are bounded only by the input's length, as for any received MemberLeaf.

**finding-5**: record — The tests that reproduce PR-jq43gx did not name it in their `verifies:` lines.
disposition: PR-jq43gx was added to the `verifies:` lines of both reproductions.

**finding-6**: record — The roadmap's never-again bullet still said a returning person may rejoin with the same key, against owner ruling 5. The S1 row also omitted REQ-yyuxh8.
disposition: the bullet now says a returning person gets a new identity, and REQ-yyuxh8 was added to the S1 row.

**finding-7**: record — RC-qa2758 was named by no test, while its sibling control RC-j2znx8 was.
disposition: RC-qa2758 was added to the `verifies:` line of `a_proof_next_to_a_neighbour_carries_none_of_its_data`.

## Gaps

- **Unreachable defensive code, uncovered, accepted by the owner on 2026-10-06.** These are serde's `BoundedSiblings::expecting` (postcard never asks for it), `path`'s two `InvariantViolated` arms (an internal node at depth 256, a leaf above depth 256), the error arms of `?` on in-range `at_level`/`insert`/`remove`, `bits.next()` running out of bits, and `HashesNotCalculated` in `path` after `prove_absent`'s guard.
- **Decision coverage is measured, not enforced.** The 123/124 figure comes from a nightly run. Adding a nightly branch-coverage target to the Makefile is a separate guardrails change that the owner approved on 2026-10-06.
- **org-node's five `quint run` invariants were not run** for this merge, by owner ruling. org-node is touched only by the PR-qmvj83 document.
- **The stale-root control and the proof gate** (whom a proof is given to) are not in this change. They are org-io's and org-node's, in roadmap S3 and S5. Until they exist, HAZ-adm7gv and HAZ-gwbn5n carry their stated residuals.
- **The app's svelte-check warning** about `@types/node` predates this change and is not addressed here.
