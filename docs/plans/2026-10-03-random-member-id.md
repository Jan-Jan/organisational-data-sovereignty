# Random member identifiers in org-node — Implementation Plan

**Goal:** org-node gives every member a `MemberId` drawn at random before the record is built, never computes one from a key, and refuses a first admission without a record snapshot — resolving PR-g7cfns.
**Implements:** REQ-d9g6nt (org-node). Resolves PR-g7cfns.
**Safety class:** C (org-node; org-members touched in documentation only).
**Verification:** org-node `verify_commands` (`org-node/.guardrails/config.yaml`, with the new fuzz target added in T2); org-members and app `verify_commands` as dependents/touched; `check-trace.sh` and `check-ids.sh` per unit; `check-units.sh`.

Environment for every run: `CARGO_HOME=/tmp/cargo_home_fuzz`; `QUINT_HOME=/private/tmp/claude-501/-Users-jan-jan-Coding-2-tier-access-control/bd80eff9-fbb7-41f8-abfa-388b4eefc384/scratchpad/quint_home` (never write `~/.quint`); one plain git command per shell call; never `sed` on Rust test files; worktree commits unsigned (`git -c commit.gpgsign=false commit`); no Co-Authored-By lines.

Owner rulings this plan implements (2026-10-03): the admin draws the identifier at random *before* calling org-members, so `Trie::genesis` / `add_member` stay pure functions of their inputs; no relation between a `MemberId` and any key; a re-admitted person is a new member under a new id and may bring the same keys; revocation is by `MemberId` (`delete_member`), never by key.

Baseline (a547ff3, 2026-10-03): org-node 56 passed + both fuzz targets + 5 quint invariants; org-members 194 passed / 1 ignored, quint 63; app 78 cargo + 30 vitest; all trace/id gates clean.

---

### T1 — Random `MemberId` at creation and admission

**Files touched:** `org-node/src/service.rs`, `org-node/tests/admission_sender.rs`
**Parallel:** yes (with T3)

1. **RED.** In `org-node/tests/admission_sender.rs`, reusing its `setup()` helper and its way of delivering an admission to B, add:

   ```rust
   // Normal case of REQ-d9g6nt: the founding admin's id and an admitted
   // member's id are not their keys, and differ from each other.
   // verifies: REQ-d9g6nt
   #[tokio::test(flavor = "multi_thread")]
   async fn member_ids_are_not_derived_from_keys() { /* ... */ }

   // Abnormal case of REQ-d9g6nt: admit B, revoke B by MemberId, admit the
   // SAME join request (same member key and device key) again. Re-admission
   // succeeds (owner ruling: same keys allowed), and the second id differs
   // from the first, which was deleted, and from every key.
   // verifies: REQ-d9g6nt
   #[tokio::test(flavor = "multi_thread")]
   async fn readmission_with_same_keys_gets_a_fresh_member_id() { /* ... */ }
   ```

   Assertions, exactly:
   - `member_ids_are_not_derived_from_keys`: read A's record snapshots (`svc_a.list_orgs()` / the org record's `trie_members`). The admin snapshot's `id != admin member key bytes` and `!= admin device key bytes`. The id `admit_member` returns for B `!= join_request_b.member_key` and `!= join_request_b.device_key`, and `!=` the admin's id.
   - `readmission_with_same_keys_gets_a_fresh_member_id`: `id1 = admit_member(.., &join_request_b, ..)`; `revoke_member(.., org_id, id1, ..)`; `id2 = admit_member(.., &join_request_b, ..)`, which must be `Ok`. Then `id2 != id1`, `id2 != join_request_b.member_key`, and A's record holds a member with id `id2` and no member with id `id1`. Repeat the revoke/re-admit cycle three times in total and assert that all ids are pairwise distinct. This randomised repetition is the property part.

   Run `cargo test -p org-node --features app,test-support --test admission_sender`. Expected red: the first test fails with `assert_ne!` (the id equals the member key bytes), and the second fails with `id2 == id1`.
2. **GREEN.** In `org-node/src/service.rs`:
   - Add a private helper next to the other free functions:

     ```rust
     /// A fresh `MemberId`: 32 bytes from the caller's cryptographic random
     /// source, drawn before the record is built, so `genesis`/`add_member`
     /// stay pure functions of their inputs. Never derived from a key (REQ-d9g6nt).
     fn fresh_member_id<R: RngCore + CryptoRng>(rng: &mut R) -> MemberId {
         let mut id = [0u8; 32];
         rng.fill_bytes(&mut id);
         MemberId::new(id)
     }
     ```

   - `create_organisation` (`:601`): `let admin_id = fresh_member_id(rng);`.
   - `admit_member` (`:790`): `let new_member_id = fresh_member_id(rng);`, and replace the comment at `:789`.
   - Leave `member_id_from_key` and its call in the `receive_and_verify` fallback for T2 to delete.
3. **VERIFY GREEN:** the full org-node `cargo test` line from the config passes, and the rest of the suite stays green. Any test that compares an id to a key is a finding to report, not something to edit away. The code map found none.
4. Commit: `feat(org-node): draw MemberId at random before the record is built (REQ-d9g6nt)`.

### T2 — Refuse a first admission without a snapshot; fuzz it

**Files touched:** `org-node/src/service.rs`, `org-node/Cargo.toml`, `org-node/.guardrails/config.yaml`, `org-node/tests/fuzz_first_admission_base/fuzz_target.rs` (new), `org-node/tests/fuzz_first_admission_base/corpus/` (new, if bolero needs it)
**Parallel:** no (serial, after T1 — same `service.rs`)

1. **Factor.** Move the `if let Some(ref snap_bytes) = msg.genesis_snapshot { … } else { … }` block in `receive_and_verify` (`service.rs:949-974`) into a function. While its body is unchanged, its fallback still returns `Ok`:

   ```rust
   /// The record a first admission extends: decoded from the snapshot the
   /// admin sent. A first admission without one is refused (REQ-d9g6nt).
   pub fn first_admission_base(genesis_snapshot: Option<&[u8]>) -> Result<Trie, OrgNodeError>
   ```

   Expose it to `tests/` the way the crate already exposes test-only constructors (`test-support` feature, cf. `test_fixtures.rs`). If it is gated, also gate the new `[[test]]` with `required-features = ["test-support"]`.
2. **RED.** Add `org-node/tests/fuzz_first_admission_base/fuzz_target.rs`, a bolero target in the style of `fuzz_envelope_decode/fuzz_target.rs` (`harness = false`, default 1 s budget). Register it in `org-node/Cargo.toml` and append `--test fuzz_first_admission_base` to the org-node `cargo test` line in `org-node/.guardrails/config.yaml`, adding a dated comment line beside the others. Content:

   ```rust
   // Abnormal case of REQ-d9g6nt over arbitrary input: no snapshot is
   // always refused, and arbitrary snapshot bytes never panic.
   // verifies: REQ-d9g6nt
   fn main() {
       // A first admission with no snapshot is refused, never completed from
       // a reconstructed record.
       assert!(org_node::service::first_admission_base(None).is_err(),
               "first admission without a snapshot was accepted");
       bolero::check!().for_each(|bytes: &[u8]| {
           let _ = org_node::service::first_admission_base(Some(bytes));
       });
   }
   ```

   Run it. Expected red: the assertion panics, because the fallback builds a record and returns `Ok`. If the fallback in fact returns `Err(Trie(DuplicateKey))` (org-members' key uniqueness refuses its leaf), the assertion passes on the old code. Then the RED must come from a stronger assertion: the error must be the new explicit variant or message (step 3), not `Trie(_)`. Write the stronger assertion first and watch it fail.
3. **GREEN.** Replace the fallback with an explicit error. Use an existing `OrgNodeError` variant if one fits (e.g. `MalformedDelta`). Otherwise use `OrgNodeError::Chain("first admission without a record snapshot".into())`, following the file's existing use of `Chain(format!(…))` for decode failures. Delete `member_id_from_key` and its doc-comment; `grep -n member_id_from_key org-node` must print nothing. Also stop swallowing a snapshot encode failure on the send side (`admit_member` `:826-832`, `revoke_member` `:1213-1214`, `postcard::to_allocvec(..).map(Some).unwrap_or(None)`). Propagate it as an error instead, so the sender can never emit `None` silently.
4. **VERIFY GREEN:** the org-node `cargo test` line (now 11 targets) passes, and the new target prints its iteration total without a crash.
5. Commit: `feat(org-node): refuse a first admission without a record snapshot; fuzz the base decode (REQ-d9g6nt)`.

### T3 — Narrow the re-admission rule in the docs; resolve PR-g7cfns

**Files touched:** `org-members/README.md`, `org-members/src/trie.rs` (doc-comments only), `org-members/docs/requirements/2026-08-31-org-membership.md`, `org-members/docs/risk/2026-09-02-membership-hazards.md`, `org-members/docs/problems/2026-10-03-delete-readd-keeps-key.md`, `org-node/docs/problems/2026-10-03-member-id-from-key.md`, `docs/CONTEXT.md`
**Parallel:** yes (with T1)

The ruling to carry: a re-admitted person is a new member under a fresh `MemberId` and **may** bring the same keys. Nothing granted to the old id carries over. The software keeps no revoked-key history. If the membership was revoked because a device was lost or stolen, that device still holds those keys' secrets; re-admission then needs fresh keys, which is the joiner's choice, not a check the software makes. This is residual risk accepted by the owner.
(Superseded wording: the canonical statement is in org-node/docs/requirements/2026-10-03-member-identity.md.)

1. `org-members/README.md`, item 11, last paragraph: replace "…and must not be given a key a device of their previous membership held." with the ruling above in one or two sentences. Keep "a deleted id must never be re-added" and the bullets about keys used *earlier* by a member that stays (rotation, PR-z463w5 ruling); those stand.
2. `org-members/src/trie.rs`, doc-comments of `add_member` (`:199-205`) and `delete_member` (`:210-216`): the same narrowing. "Never supply them again" becomes "a re-admitted member may bring them (a new member under a new id); where a removed device was compromised, fresh keys are the caller's choice". No code change.
3. `org-members/docs/requirements/2026-08-31-org-membership.md` `:109-118`, `org-members/docs/risk/2026-09-02-membership-hazards.md` `:215-226`, and `org-members/docs/problems/2026-10-03-delete-readd-keeps-key.md` `:44-53`: reword "not giving a re-admitted member such a key is the caller's duty" to the accepted-residual form, citing the owner ruling of 2026-10-03. Keep each item's place and ID; amendments are edited in place.
4. `org-node/docs/problems/2026-10-03-member-id-from-key.md`: set `status: resolved` and add a `resolution:` line directly under it, in the form of `2026-09-28-loopback-transport-timeout.md`. The resolution: REQ-d9g6nt — ids drawn at random before the record is built, `member_id_from_key` deleted, and the snapshot-less fallback refused. The second half of the report (same keys on rejoin) is resolved by owner ruling: allowed.
5. `docs/CONTEXT.md` `:43-46` (MemberId): add "drawn at random by the admitting administrator's node when the member is placed in the record". Keep the existing wording.
6. Run `GR_CONFIG=org-members/.guardrails/config.yaml sh .guardrails/scripts/check-trace.sh` and the same for org-node. Both must exit 0. `cargo test -p org-members --doc` must stay green.
7. Commit: `docs: a re-admitted member may keep their keys (owner ruling); PR-g7cfns resolved`.

---

## Self-review

1. REQ-d9g6nt is verified by T1's two tests (normal and abnormal) and by T2's fuzz target (abnormal, arbitrary input).
2. Each task gives real code or exact replacement text, the commands to run and the expected red.
3. The name `fresh_member_id` is used in T1 and `first_admission_base` in T2, consistently.
4. T1 and T3 share no file. T2 is serial after T1, because both touch `service.rs`.

## Progress

(red → green attestations are recorded here as dispatch reports arrive)

Baseline correction: org-node's cargo line counted **53**, not 56. The baseline report's per-target figures (23+3+3+4+3+1+13+3) sum to 53, so the 56 was an addition slip.

**T3 — done** (merged). This is documentation only, so there is no red/green. org-members check-trace exits 0. org-node check-trace's only failure was `MISSING-TEST REQ-d9g6nt` (pending T1), plus `NON-EXPORTED-REF LLR-v6gfc7` from the draft requirement. That second one was fixed on the change branch: the note no longer cites org-members' internal LLR.

**T1 — done** (merged).
- red -> green: `member_ids_are_not_derived_from_keys` — failed with "the admin's id is its member key" (`assert_ne!`) before `fresh_member_id` existed.
- red -> green: `readmission_with_same_keys_gets_a_fresh_member_id` — failed with "round 0: id is B's member key", at the first admission's key check, before its `id2 != id1` check.
- result: 55 passed, 0 failed (53 + 2). Both fuzz targets ran their budget clean.
- surprise: the test harness's `spawn_sink()` must keep its endpoint open until the send returns. Otherwise `revoke_member` waits in `send.stopped()` until the 30 s test timeout. That behaviour belongs to the sender's acknowledgement wait, not to this change.

**T2 — done** (merged).
- red -> green: `fuzz_first_admission_base` — step 2's contingency applied: the old fallback already returned `Err(Trie(DuplicateKey))`, so `is_err()` would have passed on the old code. The stronger assertion (an explicit `Chain(..)` error naming the missing snapshot) was written first. Before the refusal existed it panicked with "first admission without a snapshot refused for the wrong reason: Trie(DuplicateKey)".
- result: org-node line exit 0. Per target: lib 23, admission_sender 5, service_stories 3, store_at_rest 4, transport_handshake 3, transport_networked 1, verify_against_chain 13, wire_frame_bound 3 (55 passed, 0 failed). Three bolero targets ran their budget clean; `fuzz_first_admission_base` ran 135,602 iterations.
- `member_id_from_key` is gone from the code. Prose in the requirement draft and in PR-g7cfns still names it, describing the defect, as intended. Snapshot encode failures on the send side now propagate as errors. `first_admission_base` lives in the `app`-gated `service` module, so its `[[test]]` requires `app`.

**Deslop — done** (567d737). Changes:
- `spawn_sink` and `spawn_rogue_receive` were merged into `spawn_recv_one(seed)`.
- The two identical snapshot encodes became `encode_record_snapshot`.
- Comments were tightened, and the requirement's test pointer now names `admission_sender.rs` and the fuzz target.
- Suite 55/0, check-trace clean.
- Left alone: `clippy::panic` on three test `panic!`s. CI lints only the org-members and on-chain-client libs, so nothing gates it.

**merge-change progress** (resume here):
- Step 1: `origin/master` f5e16d2 merged.
- Impact set: org-members and org-node touched, app dependent.
- Step 3: drafts finalized to `2026-10-03-member-identity.md` (requirements and risk).
- Steps 4–5: check-ids and check-trace exit 0 in all three units.
- Tree under verification: `5912df1716a13b37d88b40972009ac3d591768e7` (HEAD 3d55f2e, clean).
- Round 1: gate green on tree 0f8a6232. Independent review verdict was "fix first": 12 findings, no defect in the behaviour itself. Mutants B2 and C3 survived.
- Fix round, code (merged):
  - red -> green: `same_persona_founding_two_organisations_gets_two_admin_ids` — under mutant B2 (admin id = member key XOR 0xff): "the same keys founding two organisations produced the same admin id".
  - red -> green: `first_admission_without_a_record_snapshot_is_refused` — under mutant C3 (old fallback re-inlined in `receive_and_verify`): "got Err(Trie(DuplicateKey))".
  - The snapshot is now encoded before the chain submit in admit and revoke (equivalent mutant, no test possible). `first_admission_base` is `#[doc(hidden)]`.
  - org-node: 57 passed, 0 failed; three bolero targets clean.
- Fix round, docs (merged): findings 4–10 and 12 corrected. service.rs line citations from `create_organisation` onward are now name citations.
- **Next (round 2):** step 2/6 gate dispatch on that tree, then 6a independent review (`.worktrees/worktree-random-member-id-review`), then the 6b record `docs/verification/2026-10-03-worktree-random-member-id.md`, then the squash for the owner to sign.
